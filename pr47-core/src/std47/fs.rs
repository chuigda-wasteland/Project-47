use std::any::TypeId;
use std::ptr::NonNull;

use tokio::fs::read_to_string;
use xjbutil::boxed_slice;

use crate::data::exception::ExceptionInner;
use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::data::Value;
use crate::ffi::async_fn::{
    AsyncFunctionBase,
    AsyncReturnType,
    AsyncShareGuard,
    AsyncVMContext,
    Promise,
    VMDataTrait
};
use crate::ffi::{DataOption, FFIException, Signature};
use crate::ffi::async_fn::value_into_ref;
use crate::vm::al31f::alloc::Alloc;

pub struct AsyncReadToStringBind();

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

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc=A>, ACTX: AsyncVMContext<VMData=VD>>(
        _context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        struct AsyncRet {
            g: AsyncShareGuard,
            result: tokio::io::Result<String>
        }

        impl<A: Alloc> AsyncReturnType<A> for AsyncRet {
            fn is_err(&self) -> bool {
                self.result.is_err()
            }

            fn resolve(self, alloc: &mut A, dests: &[*mut Value]) -> Result<usize, ExceptionInner> {
                match self.result {
                    Ok(data) => {
                        let value: Value = Value::new_owned(data);
                        unsafe {
                            alloc.add_managed(value.ptr_repr);
                            **dests.get_unchecked(0) = value;
                        }
                        Ok(1)
                    }
                    Err(e) => {
                        let err_value: Value = Value::new_owned(e);
                        unsafe {
                            alloc.add_managed(err_value.ptr_repr);
                        }
                        Err(ExceptionInner::Checked(err_value))
                    }
                }
            }
        }

        let (r, g) = value_into_ref::<String>(*args.get_unchecked(0))?;

        let fut = async move {
            let result = read_to_string(r).await;
            Box::new(AsyncRet { g, result }) as Box<dyn AsyncReturnType<A>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const ASYNC_READ_TO_STRING_BIND: &'static AsyncReadToStringBind = &AsyncReadToStringBind();
