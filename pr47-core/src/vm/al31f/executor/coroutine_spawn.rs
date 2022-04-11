use std::mem::transmute;
use std::ptr::NonNull;

use xjbutil::unchecked::{UncheckedSendFut, UncheckedSendSync};

use crate::data::exception::{ExceptionInner, UncheckedException};
use crate::data::Value;
use crate::ffi::async_fn::{AsyncReturnType, Promise};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::CompiledProgram;
use crate::vm::al31f::exception::Exception;
use crate::vm::al31f::executor::VMThread;
use crate::vm::al31f::executor::{create_vm_child_thread, vm_thread_run_function};
use crate::vm::al31f::stack::StackSlice;

#[cfg(feature = "async-astd")] use std::convert::Infallible as JoinError;
#[cfg(feature = "async-astd")] use async_std::task::JoinHandle;
#[cfg(feature = "async-tokio")] use futures::TryFutureExt;
#[cfg(feature = "async-tokio")] use tokio::task::{JoinError, JoinHandle};

#[inline(never)]
pub unsafe fn coroutine_spawn<A: Alloc>(
    thread: &mut VMThread<A>,
    slice: &mut StackSlice,
    func_id: usize,
    args: &[usize]
) -> Promise<A> {
    pub struct ResetPtr(*mut bool);

    impl Drop for ResetPtr {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { *self.0 = false; }
            }
        }
    }

    pub struct AsyncRet {
        ret: Result<Vec<Value>, Exception>,
        #[allow(dead_code)]
        pinned: ResetPtr
    }

    impl AsyncRet {
        pub fn new_in<A: Alloc>(ret: Result<Vec<Value>, Exception>, alloc: &mut A) -> Self {
            let pinned: *mut bool = match &ret {
                Ok(values) => unsafe { alloc.pin_objects(values) },
                Err(e) => match e.inner {
                    ExceptionInner::Checked(e) => unsafe { alloc.pin_objects(&[e]) },
                    _ => std::ptr::null_mut()
                }
            };

            Self { ret, pinned: ResetPtr(pinned) }
        }

        pub unsafe fn new_unchecked_exc(unchecked: Exception) -> Self {
            Self {
                ret: Err(unchecked),
                pinned: ResetPtr(std::ptr::null_mut())
            }
        }
    }

    impl<A: Alloc> AsyncReturnType<A> for AsyncRet {
        fn is_err(&self) -> bool {
            self.ret.is_err()
        }

        fn resolve(
            self: Box<Self>,
            _alloc: &mut A,
            dests: &[*mut Value]
        ) -> Result<usize, ExceptionInner> {
            match self.ret {
                Ok(values) => {
                    let len: usize = values.len();
                    for i in 0..len {
                        unsafe { **dests.get_unchecked(i) = *values.get_unchecked(i); }
                    }
                    Ok(len)
                },
                Err(e) => Err(e.inner)
            }
        }
    }

    unsafe impl Send for AsyncRet {}
    unsafe impl Sync for AsyncRet {}

    pub struct AsyncRet2<A: Alloc> {
        result: Result<Promise<A>, JoinError>
    }

    impl<A: Alloc> AsyncRet2<A> {
        pub fn new(join_handle: Promise<A>) -> Self {
            Self { result: Ok(join_handle) }
        }
    }

    impl<A: Alloc> AsyncReturnType<A> for AsyncRet2<A> {
        fn is_err(&self) -> bool {
            self.result.is_err()
        }

        fn resolve(
            self: Box<Self>,
            alloc: &mut A,
            dests: &[*mut Value]
        ) -> Result<usize, ExceptionInner> {
            match self.result {
                Ok(join_handle) => {
                    let join_handle_value: Value = Value::new_owned(join_handle);
                    unsafe {
                        alloc.add_managed(join_handle_value);
                        **dests.get_unchecked(0) = join_handle_value;
                    }
                    Ok(1)
                },
                Err(e) => Err(ExceptionInner::Unchecked(UncheckedException::JoinError { inner: e }))
            }
        }
    }

    unsafe impl<A: Alloc> Send for AsyncRet2<A> {}
    unsafe impl<A: Alloc> Sync for AsyncRet2<A> {}

    let thread: &'static mut VMThread<A> = transmute::<_, _>(thread);
    let args: Box<[Value]> = args.iter().map(|arg: &usize| slice.get_value(*arg)).collect();
    let program: NonNull<CompiledProgram<A>> = thread.program;
    let arg_pack: UncheckedSendSync<_> = UncheckedSendSync::new((args, program));

    let get_join_handle = async move {
        let join_handle: JoinHandle<Box<dyn AsyncReturnType<A>>> = thread.vm.co_spawn_task(
            |child_context, (func_id, arg_pack)| UncheckedSendFut::new(async move {
                let (args, program): (Box<[Value]>, NonNull<CompiledProgram<A>>) =
                    arg_pack.into_inner();
                let mut new_thread: Box<VMThread<A>> =
                    create_vm_child_thread(child_context, program);
                let arg_pack = UncheckedSendSync::new(
                    (new_thread.as_mut(), func_id, args.as_ref())
                );

                match vm_thread_run_function::<_, false>(arg_pack) {
                    Ok(f) => Box::new(AsyncRet::new_in(
                        f.await.into_inner(),
                        &mut new_thread.vm.get_shared_data_mut().alloc
                    )) as _,
                    Err(err) => Box::new(AsyncRet::new_unchecked_exc(err)) as _
                }
            }),
            (func_id, arg_pack)
        ).await;

        #[cfg(feature = "async-tokio")]
        let join_handle = join_handle.map_ok_or_else(
            |e| Box::new(AsyncRet::new_unchecked_exc(Exception::unchecked_exc(
                UncheckedException::JoinError { inner: e }
            ))) as _,
            |data| data
        );

        let join_handle_promise: Promise<A> = Promise(Box::pin(join_handle));
        Box::new(AsyncRet2::new(join_handle_promise)) as Box<dyn AsyncReturnType<A>>
    };

    Promise(Box::pin(get_join_handle))
}
