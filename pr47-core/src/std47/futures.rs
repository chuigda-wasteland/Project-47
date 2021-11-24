use std::future::Future;
use std::pin::Pin;
use futures::future::select_all;
use smallvec::{SmallVec, smallvec};
use xjbutil::boxed_slice;
use xjbutil::async_utils::join_all;

use crate::data::tyck::TyckInfoPool;
use crate::data::Value;
use crate::ffi::async_fn::{
    AsyncFunctionBase,
    AsyncReturnType,
    AsyncVMContext,
    Promise,
    PromiseContext,
    PromiseGuard,
    VMDataTrait
};
use crate::ffi::async_fn::{value_move_out_check_norm_noalias, value_move_out_norm_noalias};
use crate::ffi::{FFIException, Signature};
use crate::util::serializer::{CoroutineSharedData, Serializer, SerializerLock};
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
        for arg in args {
            value_move_out_check_norm_noalias(*arg)?;
        }

        let mut ctx: SmallVec<[PromiseContext<A>; 4]> = smallvec![];
        let mut futs: SmallVec<[Pin<Box<dyn Future<Output = AsyncReturnType> + Send>>; 4]>
            = smallvec![];

        for promise in args.into_iter()
            .map(|arg: &Value| value_move_out_norm_noalias::<Promise<A>>(*arg))
        {
            ctx.push(promise.ctx);
            futs.push(promise.fut);
        }

        let serializer: Serializer<(CoroutineSharedData, ACTX::VMData)> =
            context.serializer().clone();

        let fut = async move {
            let results: Vec<AsyncReturnType> = join_all(futs).await;
            let mut serialized: SerializerLock<(CoroutineSharedData, ACTX::VMData)> =
                serializer.lock().await;
            let alloc: &mut <ACTX::VMData as VMDataTrait>::Alloc = serialized.1.get_alloc();

            let x = results.into_iter().zip(ctx.into_iter())
                .map(|(result, ctx): (AsyncReturnType, _)| {
                    match result.0 {
                        Ok(values) => {
                            if let Some(resolver) = ctx.resolver {
                                resolver(alloc, &values);
                            }
                            Ok(values)
                        },
                        Err(e) => Err(e),
                    }
                }).fold(Ok(vec![]), |collected, incoming| {
                    if let Ok(mut values) = collected {
                        values.extend_from_slice(&incoming?);
                        Ok(values)
                    } else {
                        collected
                    }
                }).map(|vec| vec.into_boxed_slice());
            AsyncReturnType(x)
        };

        Ok(Promise {
            fut: Box::pin(fut),
            ctx: PromiseContext {
                guard: PromiseGuard { guards: boxed_slice![], reset_guard_count: 0 },
                resolver: None
            }
        })
    }
}

pub struct SelectBind();

impl AsyncFunctionBase for SelectBind {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        unimplemented!("select operation does not have standard signature")
    }

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc=A>, ACTX: AsyncVMContext<VMData=VD>>(
        context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        for arg in args {
            value_move_out_check_norm_noalias(*arg)?;
        }

        let mut ctx: SmallVec<[PromiseContext<A>; 4]> = smallvec![];
        let mut futs: SmallVec<[Pin<Box<dyn Future<Output = AsyncReturnType> + Send>>; 4]>
            = smallvec![];

        for promise in args.into_iter()
            .map(|arg: &Value| value_move_out_norm_noalias::<Promise<A>>(*arg))
        {
            ctx.push(promise.ctx);
            futs.push(promise.fut);
        }

        let serializer: Serializer<(CoroutineSharedData, ACTX::VMData)> =
            context.serializer().clone();

        let fut = async move {
            let (result, idx, _rest) = select_all(futs).await;
            let mut serialized: SerializerLock<(CoroutineSharedData, ACTX::VMData)> =
                serializer.lock().await;
            let alloc: &mut <ACTX::VMData as VMDataTrait>::Alloc = serialized.1.get_alloc();

            match result.0 {
                Ok(data) => {
                    if let Some(resolver) = ctx[idx].resolver {
                        resolver(alloc, &data);
                    }
                    let mut values: Vec<Value> = vec![Value::new_int(idx as i64)];
                    values.extend_from_slice(&data);
                    AsyncReturnType(Ok(values.into_boxed_slice()))
                },
                Err(e) => AsyncReturnType(Err(e))
            }
        };

        Ok(Promise {
            fut: Box::pin(fut),
            ctx: PromiseContext {
                guard: PromiseGuard { guards: boxed_slice![], reset_guard_count: 0 },
                resolver: None
            }
        })
    }
}

pub struct SleepBind();

impl AsyncFunctionBase for SleepBind {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
        todo!()
    }

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc=A>, ACTX: AsyncVMContext<VMData=VD>>(
        context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        todo!()
    }
}
