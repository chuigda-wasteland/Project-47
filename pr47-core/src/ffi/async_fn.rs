use std::future::Future;
use std::pin::Pin;

use crate::data::Value;
use crate::data::traits::{StaticBase};
use crate::data::wrapper::Wrapper;
use crate::ffi::{FFIException, Signature};
use crate::util::serializer::Serializer;
use crate::util::void::Void;

pub trait AsyncVMContext: 'static + Sized + Send + Sync {
    type SharedData;

    fn serializer(&self) -> &Serializer<Self::SharedData>;
}

impl StaticBase<Promise> for Void {
    fn type_name() -> String {
        "promise".to_string()
    }
}

pub trait AsyncFunctionBase: 'static {
    fn signature() -> Signature;

    fn call_tyck<ACTX: AsyncVMContext>(context: &ACTX, args: &[Value]) -> Promise;

    unsafe fn call_rtlc<ACTX: AsyncVMContext>(context: &ACTX, args: &[Value]) -> Promise;
}

pub trait AsyncFunction<ACTX: AsyncVMContext>: 'static {
    fn signature(&self) -> Signature;

    fn call_tyck(&self, context: &ACTX, args: &[Value]) -> Promise;

    unsafe fn call_rtlc(&self, context: &ACTX, args: &[Value]) -> Promise;
}

impl<AFBase, CTX> AsyncFunction<CTX> for AFBase where
    AFBase: AsyncFunctionBase,
    CTX: AsyncVMContext
{
    fn signature(&self) -> Signature {
        <AFBase as AsyncFunctionBase>::signature()
    }

    fn call_tyck(&self, context: &CTX, args: &[Value]) -> Promise {
        <AFBase as AsyncFunctionBase>::call_tyck(context, args)
    }

    unsafe fn call_rtlc(&self, context: &CTX, args: &[Value]) -> Promise {
        <AFBase as AsyncFunctionBase>::call_rtlc(context, args)
    }
}

#[derive(Copy, Clone)]
pub struct AsyncResetGuard {
    wrapper_ptr: *mut Wrapper<()>,
    ownership_info: u8
}

#[derive(Copy, Clone)]
pub struct AsyncShareGuard {
    wrapper_ptr: *mut Wrapper<()>
}

pub union AsyncOwnershipGuard {
    reset_guard: AsyncResetGuard,
    share_guard: AsyncShareGuard
}

impl AsyncOwnershipGuard {
    #[inline(always)]
    pub fn new_reset_guard(wrapper_ptr: *mut Wrapper<()>, ownership_info: u8) -> Self {
        Self {
            reset_guard: AsyncResetGuard {
                wrapper_ptr, ownership_info
            }
        }
    }

    #[inline(always)]
    pub fn new_share_guard(wrapper_ptr: *mut Wrapper<()>) -> Self {
        Self {
            share_guard: AsyncShareGuard { wrapper_ptr }
        }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn reset(&self) {
        unsafe {
            (*self.reset_guard.wrapper_ptr).ownership_info = self.reset_guard.ownership_info;
        }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn un_share(&self) {
        unsafe {
            (*self.share_guard.wrapper_ptr).ownership_info =
                (*self.share_guard.wrapper_ptr).ownership_info2;
        }
    }
}

pub type AsyncReturnType = Result<Box<[Value]>, FFIException>;

pub struct Promise {
    pub fut: Pin<Box<dyn Future<Output = AsyncReturnType> + Send + 'static>>,
    pub guards: Box<[AsyncOwnershipGuard]>,
    pub reset_guard_count: usize
}
