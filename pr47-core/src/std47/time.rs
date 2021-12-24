use std::any::TypeId;
use std::ptr::NonNull;
use std::time::{Duration, Instant};
use xjbutil::boxed_slice;

use xjbutil::void::Void;

use crate::data::traits::StaticBase;
use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::data::Value;
use crate::ffi::{DataOption, FFIException, Signature};
use crate::ffi::sync_fn::{FunctionBase, VMContext};

impl StaticBase<Instant> for Void {}
impl StaticBase<Duration> for Void {}

pub struct DurationForMillisBind();

impl FunctionBase for DurationForMillisBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        let i64_type: NonNull<TyckInfo> =
            tyck_info_pool.create_plain_type(TypeId::of::<i64>());
        let duration_type: NonNull<TyckInfo> =
            tyck_info_pool.create_plain_type(TypeId::of::<Duration>());

        Signature {
            func_type: tyck_info_pool.create_function_type(&[i64_type], &[duration_type], &[]),
            param_options: boxed_slice![DataOption::Copy],
            ret_option: boxed_slice![DataOption::Move]
        }
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        Self::call_unchecked(context, args, rets)
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        let millis: i64 = args.get_unchecked(0).vt_data.inner.int_value;
        let duration: Duration = Duration::from_millis(millis as u64);

        let value: Value = Value::new_owned(duration);
        context.add_heap_managed(value);
        **rets.get_unchecked(0) = value;

        Ok(())
    }
}
