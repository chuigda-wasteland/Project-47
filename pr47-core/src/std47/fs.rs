use std::any::TypeId;
use std::ptr::NonNull;

use tokio::fs::read_to_string;
use xjbutil::boxed_slice;

use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::data::Value;
use crate::ffi::async_fn::{
    AsyncFunctionBase,
    AsyncOwnershipGuard,
    AsyncReturnType,
    AsyncVMContext,
    Promise,
    PromiseContext,
    PromiseGuard,
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
        let (r, g) = value_into_ref::<String>(*args.get_unchecked(0))?;

        let fut = async move {
            AsyncReturnType(match read_to_string(r).await {
                Ok(s) => Ok(boxed_slice![Value::new_owned(s)]),
                Err(e) => Err(FFIException::Checked(Value::new_owned(e))),
            })
        };

        Ok(Promise {
            fut: Box::pin(fut),
            ctx: PromiseContext {
                guard: PromiseGuard {
                    guards: boxed_slice![AsyncOwnershipGuard { share_guard: g }],
                    reset_guard_count: 0
                },
                resolver: Some(|alloc: &mut A, values: &[Value]| {
                    alloc.add_managed(values.get_unchecked(0).ptr_repr)
                })
            }
        })
    }
}

pub const ASYNC_READ_TO_STRING_BIND: &'static AsyncReadToStringBind = &AsyncReadToStringBind();
