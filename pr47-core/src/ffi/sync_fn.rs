use crate::data::Value;
use crate::ffi::Signature;
use crate::data::exception::Exception;

pub trait VMContext: 'static + Sized + Send {
    // TODO the design has not been determined
}

pub trait Function: 'static {
    fn signature(&self) -> Signature;

    fn call_tyck(
        &self,
        context: &impl VMContext,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_rtlc(
        &self,
        context: &impl VMContext,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;

    unsafe fn call_unchecked(
        &self,
        context: &impl VMContext,
        args: &[Value],
        rets: &mut [&mut Value]
    ) -> Option<Exception>;
}
