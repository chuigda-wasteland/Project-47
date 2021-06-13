//! # `serializer.rs`: "User space", runtime unaware coroutine serializer
//!
//! This serializer could make sure only one of the participating `task`s can can run at one time,
//! no matter which runtime the user chose. However, `task`s can still benefit from asynchronous
//! completion of `Future`s.
//!
//! The purpose of making this "coroutine serializer" is that Pr47 heavily relies on
//! *Run-Time Lifetime Checking* (RTLC) and related analysis, which are hard to go multi-threading.
//! Forcing everything to happen in a single-threaded, sequential, serialized behavior would
//! solve this problem easily.

use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::future::Future;
use std::mem::{replace, transmute};
use std::sync::Arc;

use futures::future::JoinAll;

use unchecked_unwrap::UncheckedUnwrap;

use crate::util::async_utils::{Mutex, MutexGuard, join_all, oneshot, task, yield_now};
use crate::util::async_utils::oneshot::{Sender, Receiver};
use crate::util::unchecked_option::UncheckedOption;
use crate::util::unchecked_cell::UncheckedCellOps;

/// Basic context shared by multiple `Serializer`s in the same *serialization group*.
///
/// The `SharedContext` serves as the manager of tasks, task IDs and task completion signals. Read
/// documentations of methods and fields for more information.
pub struct SharedContext {
    /// Take down the task ID allocation status.
    next_task_id: u32,
    /// All running tasks. The key part is task ID, and the value part serves as a receiver of
    /// "task completion" signal.
    running_tasks: HashMap<u32, Receiver<()>>
}

impl SharedContext {
    /// Creates a new `SharedContext`.
    pub fn new() -> Self {
        Self {
            next_task_id: 1,
            running_tasks: HashMap::new()
        }
    }

    /// Add a new task to context, saving the "completion signal receiver" to the context, returning
    /// the allocated task ID.
    ///
    /// The allocated task ID starts from `1` instead of `0`, since the main task is not managed
    /// by `SharedContext`.
    pub fn add_task(&mut self, rx: Receiver<()>) -> u32 {
        let task_id: u32 = self.get_next_id();
        self.running_tasks.insert(task_id, rx);
        task_id
    }

    /// Remove the given task from context, together with its "completion signal receiver". This
    /// is called on child task exit, in order to reduce the burden of main task.
    pub fn remove_task(&mut self, task_id: u32) {
        self.running_tasks.remove(&task_id);
    }

    /// Retrieve all tasks and their "completion signal receiver", cleaning internal storage of
    /// `SharedContext`. This is used by main task to `await` for all running child tasks.
    pub fn get_all_tasks(&mut self) -> HashMap<u32, Receiver<()>> {
        replace(&mut self.running_tasks, HashMap::new())
    }

    /// Allocate one task ID.
    fn get_next_id(&mut self) -> u32 {
        let r: u32 = self.next_task_id;
        self.next_task_id += 1;
        r
    }
}

/// A `MutexGuard` guarding a unique access to `SharedContext`. Also serves as a
/// "running permission" for tasks.
type Permit = MutexGuard<'static, SharedContext>;

/// Serializer context of one task
pub struct Serializer {
    /// Context shared by all tasks in the same serialization group.
    shared: Arc<Mutex<SharedContext>>,
    /// Running permission of the current task.
    permit: UnsafeCell<UncheckedOption<Permit>>,
    /// Task ID of the current task. `0` implies main task, while other values are used for
    /// children tasks.
    pub task_id: u32
}

impl Serializer {
    pub async fn new() -> Self {
        let shared: Arc<Mutex<SharedContext>> = Arc::new(Mutex::new(SharedContext::new()));
        let permit: Permit = unsafe {
            transmute::<>(shared.lock().await)
        };
        Self {
            shared,
            permit: UnsafeCell::new(UncheckedOption::new(permit)),
            task_id: 0
        }
    }

    pub async fn co_yield(&self) {
        unsafe { drop(self.release_permit()); }
        yield_now().await;
        unsafe { self.acquire_permit().await; }
    }

    pub async fn co_await<FUT, T>(&self, fut: FUT) -> T
        where FUT: Future<Output=T>,
              T: Send + Sync
    {
        unsafe { drop(self.release_permit()); }
        let ret: T = fut.await;
        unsafe { self.acquire_permit().await; }
        ret
    }

    pub async fn co_spawn<F, ARGS, FUT, T>(&self, f: F, args: ARGS) -> task::JoinHandle<T>
        where F: (FnOnce(Serializer, ARGS) -> FUT) + Send + 'static,
              ARGS: Send + 'static,
              FUT: Future<Output=T> + Send,
              T: Send + 'static
    {
        let (tx, rx): (Sender<()>, Receiver<()>) = oneshot::channel();
        let task_id: u32 = unsafe {
            self.permit.get_mut_ref_unchecked().get_mut().add_task(rx)
        };
        let child_serializer: Serializer = unsafe { self.derive_child_serializer(task_id) };
        let x: task::JoinHandle<T> = task::spawn(async move {
            let r: T = f(child_serializer, args).await;
            let _ = tx.send(());
            r
        });
        unsafe { self.acquire_permit().await; }
        x
    }

    pub async fn finish(&self) {
        loop {
            unsafe {
                let running_tasks: HashMap<u32, Receiver<()>> =
                    self.permit.get_mut_ref_unchecked().get_mut().get_all_tasks();
                if running_tasks.len() == 0 {
                    break;
                }
                let fut: JoinAll<_> = join_all(
                    running_tasks.into_iter().map(|(_tid, rx): (u32, Receiver<()>)| async move {
                        rx.await.unchecked_unwrap()
                    })
                );
                self.co_await(fut).await;
            }
        }
    }

    unsafe fn derive_child_serializer(&self, task_id: u32) -> Serializer {
        let shared: Arc<Mutex<SharedContext>> = self.shared.clone();
        let permit: Permit = self.release_permit();
        Serializer {
            shared,
            permit: UnsafeCell::new(UncheckedOption::new(permit)),
            task_id
        }
    }

    async unsafe fn acquire_permit(&self) {
        let permit: Permit = transmute::<>(self.shared.lock().await);
        self.permit.get_mut_ref_unchecked().set(permit);
    }

    #[must_use] unsafe fn release_permit(&self) -> Permit {
        self.permit.get_mut_ref_unchecked().take()
    }
}

impl Drop for Serializer {
    fn drop(&mut self) {
        let mut permit: Permit = unsafe { self.permit.get_mut().take() };
        if self.task_id == 0 {
            assert_eq!(permit.running_tasks.len(), 0);
        } else {
            permit.remove_task(self.task_id);
        }
    }
}

unsafe impl Send for Serializer {}
unsafe impl Sync for Serializer {}

#[cfg(test)]
mod test {
    use std::time::Duration;
    use crate::util::serializer::Serializer;
    use crate::util::async_utils::{block_on_future, testing_sleep};

    #[test]
    fn basic_test_print() {
        async fn test_impl() {
            let serializer: Serializer = Serializer::new().await;
            eprintln!("line 1");
            serializer.co_spawn(|serializer: Serializer, _x: ()| async move {
                eprintln!("line 2");
                serializer.co_yield().await;
                eprintln!("line 3");
            }, ()).await;
            eprintln!("line 4");
            serializer.co_spawn(|serializer: Serializer, _x: ()| async move {
                eprintln!("line 5");
                serializer.co_yield().await;
                eprintln!("line 6");
                serializer.co_await(testing_sleep(Duration::from_millis(500))).await;
                eprintln!("line 7");
            }, ()).await;
            eprintln!("line 8");
            serializer.finish().await;
            eprintln!("line 9");
        }

        eprintln!("launching test");
        block_on_future(test_impl());
        eprintln!("mission accomplished");
    }
}
