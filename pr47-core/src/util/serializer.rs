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
use std::mem::transmute;
use std::sync::Arc;

use futures::future::JoinAll;
use xjbutil::async_utils::{Mutex, MutexGuard, join_all, oneshot, task, yield_now};
use xjbutil::async_utils::oneshot::{Receiver, Sender};
use xjbutil::unchecked::{UncheckedCellOps, UncheckedOption};

/// A `Arc<Mutex>` is basically a "serializer" context, serializing accesses to `Data`.
pub type Serializer<Data> = Arc<Mutex<Data>>;

/// A `MutexGuard` guards unique access to `Data`. Logically, this structure serves as a
/// "running permission" for coroutines/tasks.
pub type SerializerLock<'a, Data> = MutexGuard<'a, Data>;

pub async fn co_yield<'a, D>(
    serializer: &'a Serializer<D>,
    lock: SerializerLock<'a, D>
) -> SerializerLock<'a, D>
    where D: 'static
{
    drop(lock);
    yield_now().await;
    serializer.lock().await
}

pub async fn co_await<'a, D, FUT, T>(
    serializer: &'a Serializer<D>,
    lock: SerializerLock<'a, D>,
    fut: FUT
) -> (T, SerializerLock<'a, D>)
    where D: 'static,
          FUT: Future<Output=T>,
          T: Send + Sync
{
    drop(lock);
    let ret: T = fut.await;
    let lock = serializer.lock().await;
    (ret, lock)
}

pub async fn co_spawn<'a, 'b, D, FN, ARGS, FUT, RET>(
    serializer: &'b Serializer<D>,
    lock: SerializerLock<'a, D>,
    f: FN,
    args: ARGS
) -> (task::JoinHandle<RET>, SerializerLock<'b, D>)
    where D: 'static,
          FN: (FnOnce(Serializer<D>, SerializerLock<'a, D>, ARGS) -> FUT) + Send + 'static,
          ARGS: Send + 'static,
          FUT: Future<Output=RET> + Send + 'static,
          RET: Send + 'static
{
    let serializer_clone: Serializer<D> = serializer.clone();
    let join_handle: task::JoinHandle<RET> = task::spawn(f(serializer_clone, lock, args));
    let lock: SerializerLock<'b, D> = serializer.lock().await;
    (join_handle, lock)
}

/// Context shared by all coroutines in the same serialization group
pub struct CoroutineSharedData {
    /// Tracks the task ID allocation status.
    next_task_id: u32,
    /// All running tasks. The key part is task ID, and the value part serves as a receiver of
    /// "task completion" signal. Note that the main task (task_id == 0) itself is not managed
    /// by this `HashMap`
    running_tasks: HashMap<u32, Receiver<()>>
}

impl CoroutineSharedData {
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
    pub fn retrieve_all_tasks(&mut self) -> HashMap<u32, Receiver<()>> {
        std::mem::take(&mut self.running_tasks)
    }

    /// Allocate one task ID.
    fn get_next_id(&mut self) -> u32 {
        let r: u32 = self.next_task_id;
        self.next_task_id += 1;
        r
    }
}

/// Context of one coroutine/task
pub struct CoroutineContext<SerializedData: 'static + Send> {
    /// Data shared by all tasks in the same serialization group.
    pub serializer: Serializer<(CoroutineSharedData, SerializedData)>,
    /// Running permission of the current task. The `UncheckedOption` is here just for
    /// temporarily dropping and re-claiming permissions.
    permit: UnsafeCell<UncheckedOption<
        SerializerLock<'static, (CoroutineSharedData, SerializedData)>
    >>,
    /// Task ID of the current task. `0` implies main task, while other values are used for
    /// children tasks.
    pub task_id: u32
}

impl<SD: 'static + Send> CoroutineContext<SD> {
    /// Creates a new, main coroutine context with given `shared_data`
    pub async fn main_context(shared_data: SD) -> Self {
        let serializer: Serializer<(CoroutineSharedData, SD)>
            = Arc::new(Mutex::new((CoroutineSharedData::new(), shared_data)));
        let permit: SerializerLock<(CoroutineSharedData, SD)>
            = unsafe { transmute::<>(serializer.lock().await) };
        Self {
            serializer,
            permit: UnsafeCell::new(UncheckedOption::new(permit)),
            task_id: 0
        }
    }

    pub fn child_context(
        serializer: Serializer<(CoroutineSharedData, SD)>,
        permit: SerializerLock<'static, (CoroutineSharedData, SD)>,
        task_id: u32
    ) -> Self {
        Self {
            serializer,
            permit: UnsafeCell::new(UncheckedOption::new(permit)),
            task_id
        }
    }

    /// Given the fact that the permit is held, and there's not another mutable reference to the
    /// shared data, retrieve the shared data.
    pub fn get_shared_data_mut(&mut self) -> &mut SD {
        unsafe {
            let permit: &mut UncheckedOption<SerializerLock<(CoroutineSharedData, SD)>> =
                self.permit.get_mut_ref_unchecked();
            &mut permit.get_mut().1
        }
    }

    /// Interrupt current `task`, allowing other `task` to run.
    pub async fn co_yield(&self) {
        let permit: SerializerLock<'static, (CoroutineSharedData, SD)>
            = unsafe { self.release_permit() };
        let permit: SerializerLock<'_, (CoroutineSharedData, SD)>
            = co_yield(&self.serializer, permit).await;
        unsafe { self.acquire_permit(permit) }
    }

    /// Interrupt current `task`, await for given `fut`. During this time other `task`s may run.
    pub async fn co_await<FUT, RET>(&self, fut: FUT) -> RET
        where FUT: Future<Output=RET>,
              RET: Send + Sync
    {
        let permit: SerializerLock<'static, (CoroutineSharedData, SD)>
            = unsafe { self.release_permit() };
        let (ret, permit): (RET, SerializerLock<'_, (CoroutineSharedData, SD)>)
            = co_await(&self.serializer, permit, fut).await;
        unsafe { self.acquire_permit(permit) }
        ret
    }

    /// Spawn a new `task` managed by the current serialization group
    pub async fn co_spawn_task<FN, ARGS, FUT, RET>(
        &self,
        f: FN,
        args: ARGS
    ) -> task::JoinHandle<RET>
        where FN: (FnOnce(CoroutineContext<SD>, ARGS) -> FUT) + Send + 'static,
              ARGS: Send + 'static,
              FUT: Future<Output=RET> + Send,
              RET: Send + 'static
    {
        let (tx, rx): (Sender<()>, Receiver<()>) = oneshot::channel();

        let mut permit: SerializerLock<'static, (CoroutineSharedData, SD)>
            = unsafe { self.release_permit() };
        let task_id: u32 = permit.0.add_task(rx);

        let (join_handle, new_permit): (task::JoinHandle<RET>, SerializerLock<_>) = co_spawn(
            &self.serializer,
            permit,
            move |serializer, permit, ()| async move {
                let child_context: CoroutineContext<SD> = Self::child_context(
                    serializer,
                    permit,
                    task_id
                );
                let r: RET = f(child_context, args).await;
                let _ = tx.send(());
                r
            },
            ()).await;
        unsafe { self.acquire_permit(new_permit); }
        join_handle
    }

    /// Called on main `task` exit, wait for all other `task`s to finish.
    pub async fn finish(&self) {
        loop {
            unsafe {
                let running_tasks: HashMap<u32, Receiver<()>> =
                    self.permit.get_mut_ref_unchecked().get_mut().0.retrieve_all_tasks();
                if running_tasks.is_empty() {
                    break;
                }
                let fut: JoinAll<_ /*: impl Future<Output=()>*/> = join_all(
                    running_tasks.into_iter().map(|(_tid, rx): (u32, Receiver<()>)| async move {
                        rx.await.unwrap_unchecked()
                    })
                );
                self.co_await(fut).await;
            }
        }
    }

    unsafe fn acquire_permit(&self, permit: SerializerLock<'_, (CoroutineSharedData, SD)>) {
        let permit: SerializerLock<'static, (CoroutineSharedData, SD)> = transmute::<>(permit);
        self.permit.get_mut_ref_unchecked().set(permit);
    }

    #[must_use]
    unsafe fn release_permit(&self) -> SerializerLock<'static, (CoroutineSharedData, SD)> {
        self.permit.get_mut_ref_unchecked().take()
    }
}

impl<SD: 'static + Send> Drop for CoroutineContext<SD> {
    fn drop(&mut self) {
        let mut permit: SerializerLock<(CoroutineSharedData, SD)>
            = unsafe { self.permit.get_mut().take() };
        if self.task_id == 0 {
            assert_eq!(permit.0.running_tasks.len(), 0);
        } else {
            permit.0.remove_task(self.task_id);
        }
        drop(permit);
    }
}

unsafe impl<SD: 'static + Send> Send for CoroutineContext<SD> {}
unsafe impl<SD: 'static + Send> Sync for CoroutineContext<SD> {}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use xjbutil::async_utils::{block_on_future, testing_sleep};

    use crate::util::serializer::CoroutineContext;

    #[test]
    fn basic_test_print() {
        async fn test_impl() {
            let serializer: CoroutineContext<()> = CoroutineContext::main_context(()).await;
            eprintln!("line 1");
            serializer.co_spawn_task(|serializer: CoroutineContext<()>, _x: ()| async move {
                eprintln!("line 2");
                serializer.co_yield().await;
                eprintln!("line 3");
            }, ()).await;
            eprintln!("line 4");
            serializer.co_spawn_task(|serializer: CoroutineContext<()>, _x: ()| async move {
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
