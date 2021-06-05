use async_trait::async_trait;

use crate::data::Value;
use crate::data::exception::Exception;
use crate::ffi::Signature;
use crate::util::serializer::Serializer;

#[async_trait]
pub trait AsyncVMContext: 'static + Sized + Send + Sync {
    fn serializer(&self) -> &Serializer;
}

#[async_trait]
pub trait AsyncFunction: 'static {
    fn signature(&self) -> Signature;

    async fn call_tyck<'a>(
        &'a self,
        context: &'a impl AsyncVMContext,
        args: &'a [Value],
        rets: &'a mut [&'a mut Value]
    ) -> Option<Exception>;

    async unsafe fn call_rtlc<'a>(
        &'a self,
        context: &'a impl AsyncVMContext,
        args: &'a [Value],
        rets: &'a mut [&'a mut Value]
    ) -> Option<Exception>;

    async unsafe fn call_unchecked<'a>(
        &'a self,
        context: &'a impl AsyncVMContext,
        args: &'a [Value],
        rets: &'a mut [&'a mut Value]
    ) -> Option<Exception>;
}
