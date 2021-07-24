use crate::data::Value;
use crate::data::exception::Exception;
use crate::ffi::Signature;

pub trait VMContext: 'static + Sized + Send {
    // TODO the design has not been determined
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
