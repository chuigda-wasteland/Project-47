use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::time::Duration;

use futures::future::select_all;
use smallvec::SmallVec;
use xjbutil::boxed_slice;
use xjbutil::async_utils::join_all;

use crate::data::Value;
use crate::data::exception::ExceptionInner;
use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::ffi::{FFIException, Signature};
use crate::ffi::async_fn::{
    AsyncFunctionBase,
    AsyncReturnType,
    AsyncVMContext,
    Promise,
    PromiseResult,
    LockedCtx
};
use crate::ffi::async_fn::{value_move_out_check_norm_noalias, value_move_out_norm_noalias};

pub struct JoinBind();

impl AsyncFunctionBase for JoinBind {
    fn signature(
        _tyck_info_pool: &mut TyckInfoPool
    ) -> Signature {
        unimplemented!("join operation does not have standard signature")
    }

    unsafe fn call_rtlc<LC: LockedCtx, ACTX: AsyncVMContext<Locked=LC>> (
        _context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<LC>, FFIException> {
        struct AsyncRet<LC: LockedCtx> {
            results: Vec<PromiseResult<LC>>
        }

        impl<LC: LockedCtx> AsyncReturnType<LC> for AsyncRet<LC> {
            fn is_err(&self) -> bool {
                self.results.iter().any(|x| x.is_err())
            }

            fn resolve(
                self: Box<Self>,
                locked_ctx: &mut LC,
                dests: &[*mut Value]
            ) -> Result<usize, ExceptionInner> {
                let len: usize = self.results.len();

                for i in 0..len {
                    unsafe {
                        if (*self.results.get_unchecked(i)).is_err() {
                            return std::ptr::read(self.results.get_unchecked(i))
                                .resolve(locked_ctx, &[]);
                        }
                    }
                }

                let mut resolved_size: usize = 0;
                for result in self.results {
                    resolved_size += unsafe {
                        result.resolve(locked_ctx, &dests[resolved_size..]).unwrap_unchecked()
                    }
                }
                Ok(resolved_size)
            }
        }

        for arg in args {
            value_move_out_check_norm_noalias(*arg)?;
        }

        let futs: SmallVec<[Pin<Box<dyn Future<Output=PromiseResult<LC>> + Send>>; 4]> =
            args.iter()
                .map(|arg: &Value| value_move_out_norm_noalias::<Promise<LC>>(*arg))
                .map(|Promise(fut)| fut)
                .collect();

        let fut = async move {
            let results: Vec<PromiseResult<LC>> = join_all(futs).await;
            Box::new(AsyncRet { results }) as Box<dyn AsyncReturnType<LC>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const JOIN_BIND: &JoinBind = &JoinBind();

pub struct SelectBind();

impl AsyncFunctionBase for SelectBind {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        unimplemented!("select operation does not have standard signature")
    }

    unsafe fn call_rtlc<LC: LockedCtx, ACTX: AsyncVMContext<Locked=LC>>(
        _context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<LC>, FFIException> {
        struct AsyncRet<LC: LockedCtx> {
            result: Box<dyn AsyncReturnType<LC>>,
            idx: usize
        }

        impl<LC: LockedCtx> AsyncReturnType<LC> for AsyncRet<LC> {
            fn is_err(&self) -> bool {
                self.result.is_err()
            }

            fn resolve(
                self: Box<Self>,
                locked_ctx: &mut LC,
                dests: &[*mut Value]
            ) -> Result<usize, ExceptionInner> {
                unsafe {
                    **dests.get_unchecked(0) = Value::new_int(self.idx as i64);
                }
                self.result.resolve(locked_ctx, &dests[1..])
            }
        }

        for arg in args {
            value_move_out_check_norm_noalias(*arg)?;
        }

        let futs: SmallVec<[Pin<Box<dyn Future<Output=PromiseResult<LC>> + Send>>; 4]> =
            args.iter()
                .map(|arg: &Value| value_move_out_norm_noalias::<Promise<LC>>(*arg))
                .map(|Promise(fut)| fut)
                .collect();

        let fut = async move {
            let (result, idx, _rest) = select_all(futs).await;
            Box::new(AsyncRet { result, idx }) as Box<dyn AsyncReturnType<LC>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const SELECT_BIND: &SelectBind = &SelectBind();

pub struct SleepMillisBind();

impl AsyncFunctionBase for SleepMillisBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        let i64_type: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(TypeId::of::<i64>());

        Signature {
            func_type: tyck_info_pool.create_function_type(&[i64_type], &[], &[]),
            param_options: boxed_slice![],
            ret_option: boxed_slice![]
        }
    }

    unsafe fn call_rtlc<LC: LockedCtx, ACTX: AsyncVMContext<Locked=LC>>(
        _context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<LC>, FFIException> {
        struct AsyncRet();

        impl<LC: LockedCtx> AsyncReturnType<LC> for AsyncRet {
            fn is_err(&self) -> bool {
                false
            }

            fn resolve(self: Box<Self>, _locked_ctx: &mut LC, _dests: &[*mut Value])
                -> Result<usize, ExceptionInner>
            {
                Ok(0)
            }
        }

        let int_value: i64 = args.get_unchecked(0).vt_data.inner.int_value;
        let fut = async move {
            #[cfg(feature = "async-astd")]
            async_std::task::sleep(Duration::from_millis(int_value as u64)).await;
            #[cfg(feature = "async-tokio")]
            tokio::time::sleep(Duration::from_millis(int_value as u64)).await;
            Box::new(AsyncRet()) as Box<dyn AsyncReturnType<LC>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const SLEEP_MS_BIND: &SleepMillisBind = &SleepMillisBind();
