//! ## `serializer.rs`: "User space", runtime unaware coroutine serializer
//!
//! This serializer could make sure only one of the participating `task`s can can run at one time,
//! no matter which runtime the user chose.

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

pub struct SharedContext {
    next_task_id: u32,
    running_tasks: HashMap<u32, Receiver<()>>
}

impl SharedContext {
    pub fn new() -> Self {
        Self {
            next_task_id: 0,
            running_tasks: HashMap::new()
        }
    }

    pub fn get_next_id(&mut self) -> u32 {
        let r: u32 = self.next_task_id;
        self.next_task_id += 1;
        r
    }

    pub fn add_task(&mut self, rx: Receiver<()>) -> u32 {
        let task_id: u32 = self.get_next_id();
        self.running_tasks.insert(task_id, rx);
        task_id
    }

    pub fn remove_task(&mut self, task_id: u32) {
        self.running_tasks.remove(&task_id);
    }

    pub fn get_all_tasks(&mut self) -> HashMap<u32, Receiver<()>> {
        replace(&mut self.running_tasks, HashMap::new())
    }
}

type Permit = MutexGuard<'static, SharedContext>;

pub struct Serializer {
    shared: Arc<Mutex<SharedContext>>,
    permit: UnsafeCell<UncheckedOption<Permit>>,
    pub task_id: u32
}

impl Serializer {
    pub async fn new() -> Self {
        let shared: Arc<Mutex<SharedContext>> = Arc::new(Mutex::new(SharedContext::new()));
        let mut permit: Permit = unsafe {
            transmute::<>(shared.lock().await)
        };
        let task_id: u32 = permit.get_next_id();
        Self {
            shared,
            permit: UnsafeCell::new(UncheckedOption::new(permit)),
            task_id
        }
    }

    pub async fn co_yield(&mut self) {
        unsafe { drop(self.release_permit()); }
        yield_now().await;
        unsafe { self.acquire_permit().await; }
    }

    pub async fn co_await<FUT, T>(&mut self, fut: FUT) -> T
        where FUT: Future<Output=T>,
              T: Send + Sync
    {
        unsafe { drop(self.release_permit()); }
        let ret: T = fut.await;
        unsafe { self.acquire_permit().await; }
        ret
    }

    pub async fn co_spawn<F, ARGS, FUT, T>(&mut self, f: F, args: ARGS) -> task::JoinHandle<T>
        where F: (FnOnce(&mut Serializer, ARGS) -> FUT) + Send + 'static,
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
            let mut child_serializer: Serializer = child_serializer;
            let r: T = f(&mut child_serializer, args).await;
            unsafe { tx.send(()).unchecked_unwrap(); }
            r
        });
        unsafe { self.acquire_permit().await; }
        x
    }

    pub async fn finish(&mut self) {
        loop {
            unsafe {
                let running_tasks: HashMap<u32, Receiver<()>> =
                    self.permit.get_mut_ref_unchecked().get_mut().get_all_tasks();
                let fut: JoinAll<_> = join_all(
                    running_tasks.into_iter().map(|(_tid, rx): (u32, Receiver<()>)| async move {
                        rx.await.unchecked_unwrap()
                    })
                );
                self.co_await(fut).await;
            }
        }
    }

    unsafe fn derive_child_serializer(&mut self, task_id: u32) -> Serializer {
        let shared: Arc<Mutex<SharedContext>> = self.shared.clone();
        let permit: Permit = self.release_permit();
        Serializer {
            shared,
            permit: UnsafeCell::new(UncheckedOption::new(permit)),
            task_id
        }
    }

    async unsafe fn acquire_permit(&mut self) {
        self.permit.get_mut().set(transmute::<>(self.shared.lock().await))
    }

    #[must_use] unsafe fn release_permit(&mut self) -> Permit {
        self.permit.get_mut().take()
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
