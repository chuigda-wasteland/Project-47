#![allow(unused_imports)]

use std::any::TypeId;
use std::ptr::NonNull;

use xjbutil::boxed_slice;

use crate::data::Value;
use crate::data::exception::ExceptionInner;
use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::ffi::{DataOption, FFIException, Signature};
use crate::vm::al31f::alloc::Alloc;

#[cfg(feature = "async")]
use crate::ffi::async_fn::{
    AsyncFunctionBase,
    AsyncReturnType,
    AsyncShareGuard,
    AsyncVMContext,
    Promise,
    LockedCtx,
    value_into_ref
};
#[cfg(feature = "async-astd")] use async_std::fs::read_to_string;
#[cfg(feature = "async-tokio")] use tokio::fs::read_to_string;

#[cfg(feature = "async")]
pub struct AsyncReadToStringBind();

#[cfg(feature = "async")]
impl AsyncFunctionBase for AsyncReadToStringBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        let string_type: NonNull<TyckInfo> = tyck_info_pool.get_string_type();
        let io_exception_type: NonNull<TyckInfo> =
            tyck_info_pool.create_plain_type(TypeId::of::<std::io::Error>());
        Signature {
            func_type: tyck_info_pool.create_function_type(
                &[string_type], &[string_type], &[io_exception_type]
            ),
            param_options: boxed_slice![DataOption::Share],
            ret_option: boxed_slice![DataOption::Move]
        }
    }

    unsafe fn call_rtlc<LC: LockedCtx, ACTX: AsyncVMContext<Locked=LC>>(
        _context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<LC>, FFIException> {
        struct AsyncRet {
            #[allow(dead_code)]
            g: AsyncShareGuard,

            #[cfg(feature = "async-astd")]
            result: async_std::io::Result<String>,
            #[cfg(feature = "async-tokio")]
            result: tokio::io::Result<String>
        }

        impl<LC: LockedCtx> AsyncReturnType<LC> for AsyncRet {
            fn is_err(&self) -> bool {
                self.result.is_err()
            }

            fn resolve(
                self: Box<Self>,
                locked_ctx: &mut LC,
                dests: &[*mut Value]
            ) -> Result<usize, ExceptionInner> {
                match self.result {
                    Ok(data) => {
                        let value: Value = Value::new_owned(data);
                        locked_ctx.add_heap_managed(value);
                        unsafe {
                            **dests.get_unchecked(0) = value;
                        }
                        Ok(1)
                    }
                    Err(e) => {
                        let err_value: Value = Value::new_owned(e);
                        locked_ctx.add_heap_managed(err_value);
                        Err(ExceptionInner::Checked(err_value))
                    }
                }
            }
        }

        let (r, g) = value_into_ref::<String>(*args.get_unchecked(0))?;

        let fut = async move {
            let result = read_to_string(r).await;
            Box::new(AsyncRet { g, result }) as Box<dyn AsyncReturnType<LC>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

#[cfg(feature = "async")]
pub const ASYNC_READ_TO_STRING_BIND: &AsyncReadToStringBind = &AsyncReadToStringBind();
