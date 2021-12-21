use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::time::Duration;
use futures::future::select_all;
use smallvec::SmallVec;
use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::boxed_slice;
use xjbutil::async_utils::join_all;
use crate::data::exception::ExceptionInner;

use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::data::Value;
use crate::ffi::async_fn::{AsyncFunctionBase, AsyncReturnType, AsyncVMContext, Promise, PromiseResult, VMDataTrait};
use crate::ffi::async_fn::{value_move_out_check_norm_noalias, value_move_out_norm_noalias};
use crate::ffi::{FFIException, Signature};
use crate::util::serializer::{CoroutineSharedData, Serializer};
use crate::vm::al31f::alloc::Alloc;

pub struct JoinBind();

impl AsyncFunctionBase for JoinBind {
    fn signature(
        _tyck_info_pool: &mut TyckInfoPool
    ) -> Signature {
        unimplemented!("join operation does not have standard signature")
    }

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc=A>, ACTX: AsyncVMContext<VMData=VD>> (
        context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        struct AsyncRet<A: Alloc> {
            results: Vec<PromiseResult<A>>
        }

        impl<A: Alloc> AsyncReturnType<A> for AsyncRet<A> {
            fn is_err(&self) -> bool {
                self.results.iter().any(|x| x.is_err())
            }

            fn resolve(self, alloc: &mut A, dests: &[*mut Value]) -> Result<usize, ExceptionInner> {
                for result in self.results {
                    if result.is_err() {
                        return result.resolve(alloc, &[])
                    }
                }

                let mut resolved_size: usize = 0;
                for result in self.results {
                    resolved_size += unsafe {
                        result.resolve(alloc, &dests[resolved_size..]).unchecked_unwrap()
                    }
                }
                Ok(resolved_size)
            }
        }

        for arg in args {
            value_move_out_check_norm_noalias(*arg)?;
        }

        let mut futs: SmallVec<[Pin<Box<dyn Future<Output=PromiseResult<A>> + Send>>; 4]> =
            args.into_iter()
                .map(|arg: &Value| value_move_out_norm_noalias::<Promise<A>>(*arg))
                .map(|Promise(fut)| fut)
                .collect();

        let serializer: Serializer<(CoroutineSharedData, ACTX::VMData)> =
            context.serializer().clone();

        let fut = async move {
            let results: Vec<PromiseResult<A>> = join_all(futs).await;
            Box::new(AsyncRet { results }) as Box<dyn AsyncReturnType<A>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const JOIN_BIND: &'static JoinBind = &JoinBind();

pub struct SelectBind();

impl AsyncFunctionBase for SelectBind {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        unimplemented!("select operation does not have standard signature")
    }

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc=A>, ACTX: AsyncVMContext<VMData=VD>>(
        context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        struct AsyncRet<A: Alloc> {
            result: Box<dyn AsyncReturnType<A>>,
            idx: usize
        }

        impl<A: Alloc> AsyncReturnType<A> for AsyncRet<A> {
            fn is_err(&self) -> bool {
                self.result.is_err()
            }

            fn resolve(self, alloc: &mut A, dests: &[*mut Value]) -> Result<usize, ExceptionInner> {
                unsafe {
                    **dests.get_unchecked(0) = Value::new_int(self.idx as i64);
                }
                self.result.resolve(alloc, &dests[1..])
            }
        }

        for arg in args {
            value_move_out_check_norm_noalias(*arg)?;
        }

        let futs: SmallVec<[Pin<Box<dyn Future<Output=PromiseResult<A>> + Send>>; 4]> =
            args.into_iter()
                .map(|arg: &Value| value_move_out_norm_noalias::<Promise<A>>(*arg))
                .map(|Promise(fut)| fut)
                .collect();

        let serializer: Serializer<(CoroutineSharedData, ACTX::VMData)> =
            context.serializer().clone();

        let fut = async move {
            let (result, idx, _rest) = select_all(futs).await;
            Box::new(AsyncRet { result, idx }) as Box<dyn AsyncReturnType<A>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const SELECT_BIND: &'static SelectBind = &SelectBind();

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

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc=A>, ACTX: AsyncVMContext<VMData=VD>>(
        _context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        struct AsyncRet();

        impl<A: Alloc> AsyncReturnType<A> for AsyncRet {
            fn is_err(&self) -> bool {
                false
            }

            fn resolve(self, _alloc: &mut A, _dests: &[*mut Value])
                -> Result<usize, ExceptionInner>
            {
                Ok(0)
            }
        }

        let int_value: i64 = args.get_unchecked(0).vt_data.inner.int_value;
        let fut = async move {
            tokio::time::sleep(Duration::from_millis(int_value as u64)).await;
            Box::new(AsyncRet()) as Box<dyn AsyncReturnType<A>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

pub const SLEEP_MS_BIND: &'static SleepMillisBind = &SleepMillisBind();
