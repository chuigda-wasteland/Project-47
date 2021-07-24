use crate::data::Value;
use crate::data::exception::Exception;
use crate::ffi::Signature;

pub trait VMContext: 'static + Sized + Send {
    // TODO the design has not been determined
}

pub trait FunctionBase: 'static {
    fn signature() -> Signature;

    fn call_tyck<CTX: VMContext>(
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_rtlc<CTX: VMContext>(
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_unchecked<CTX: VMContext>(
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;
}

pub trait Function<CTX: VMContext>: 'static {
    fn signature(&self) -> Signature;

    fn call_tyck(
        &self,
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_rtlc(
        &self,
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_unchecked(
        &self,
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;
}

impl<FBase, CTX> Function<CTX> for FBase where
    FBase: FunctionBase,
    CTX: VMContext
{
    #[inline] fn signature(&self) -> Signature {
        <FBase as FunctionBase>::signature()
    }

    #[inline] fn call_tyck(
        &self,
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception> {
        <FBase as FunctionBase>::call_tyck(context, args, rets)
    }

    #[inline] unsafe fn call_rtlc(
        &self,
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception> {
        <FBase as FunctionBase>::call_rtlc(context, args, rets)
    }

    #[inline] unsafe fn call_unchecked(
        &self,
        context: &CTX,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception> {
        <FBase as FunctionBase>::call_unchecked(context, args, rets)
    }
}
