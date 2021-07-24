use std::future::Future;
use std::pin::Pin;

use crate::data::Value;
use crate::data::exception::Exception;
use crate::data::traits::StaticBase;
use crate::data::wrapper::OwnershipInfo;
use crate::ffi::Signature;
use crate::util::serializer::Serializer;
use crate::util::void::Void;

pub enum AsyncOwnInfoGuard {
    DoNothing,
    SetOwnInfo(Value, OwnershipInfo)
}

pub trait AsyncVMContext: 'static + Sized + Send + Sync {
    type SharedData;

    fn serializer(&self) -> &Serializer<Self::SharedData>;
}

pub type AsyncReturnType = Result<Box<[Value]>, Exception>;

pub struct Promise {
    pub fut: Pin<Box<dyn Future<Output = AsyncReturnType> + Send + 'static>>,
    pub guards: Box<[AsyncOwnInfoGuard]>
}

// TODO should we make it a `StaticBase`?
impl StaticBase<Promise> for Void {}

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
