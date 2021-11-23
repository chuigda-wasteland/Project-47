use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::task::{Context, Poll};

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::either::Either;
use xjbutil::unchecked::UncheckedSendSync;
use xjbutil::wide_ptr::WidePointer;

use crate::builtins::object::Object;
use crate::data::Value;
use crate::data::exception::{CheckedException, Exception, UncheckedException};
use crate::data::value_typed::INT_TYPE_TAG;
use crate::ffi::sync_fn::Function as FFIFunction;
use crate::vm::al31f::{AL31F, Combustor};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::executor::checked_bin_ops::{
    checked_add,
    checked_bit_and,
    checked_bit_or,
    checked_bit_shl,
    checked_bit_shr,
    checked_bit_xor,
    checked_div,
    checked_ge,
    checked_gt,
    checked_le,
    checked_logic_and,
    checked_logic_or,
    checked_lt,
    checked_mod,
    checked_mul,
    checked_sub
};
use crate::vm::al31f::executor::checked_cast_ops::{
    cast_any_bool,
    cast_any_char,
    cast_any_float,
    cast_any_int
};
use crate::vm::al31f::executor::checked_unary_ops::{
    checked_bit_not,
    checked_neg,
    checked_not
};
use crate::vm::al31f::insc::Insc;
use crate::vm::al31f::stack::{FrameInfo, Stack, StackSlice};

#[cfg(feature = "async")] use std::hint::unreachable_unchecked;
#[cfg(feature = "async")] use std::mem::transmute;
#[cfg(feature = "async")] use futures::FutureExt;
#[cfg(feature = "async")] use crate::data::wrapper::{Wrapper, OwnershipInfo};
#[cfg(feature = "async")] use crate::ffi::async_fn::{AsyncReturnType, Promise};
#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncFunction as FFIAsyncFunction;
#[cfg(feature = "async")] use crate::util::serializer::CoroutineContext;
#[cfg(feature = "async")] use crate::vm::al31f::AsyncCombustor;

include!("get_vm_makro.rs");
include!("impl_makro.rs");

pub struct VMThread<A: Alloc> {
    #[cfg(feature = "async")]
    pub vm: CoroutineContext<AL31F<A>>,
    #[cfg(not(feature = "async"))]
    pub vm: AL31F<A>,

    pub program: NonNull<CompiledProgram<A>>,
    pub stack: Stack
}

#[must_use = "VM thread are effective iff a function gets run on it"]
#[cfg(feature = "async")]
pub async fn create_vm_main_thread<A: Alloc>(
    alloc: A,
    program: &CompiledProgram<A>
) -> Box<VMThread<A>> {
    let mut ret = Box::new(VMThread {
        vm: CoroutineContext::main_context(AL31F::new(alloc)).await,
        program: NonNull::from(program),
        stack: Stack::new()
    });
    unsafe { ret.vm.get_shared_data_mut().alloc.add_stack(&ret.stack) };
    ret
}

unsafe fn unchecked_exception_unwind_stack(
    unchecked_exception: UncheckedException,
    stack: &mut Stack,
    insc_ptr: usize
) -> Exception {
    let mut exception: Exception = Exception::unchecked_exc(unchecked_exception);

    let mut insc_ptr: usize = insc_ptr;
    while stack.frames.len() != 0 {
        let last_frame: &FrameInfo = stack.frames.last().unchecked_unwrap();
        exception.push_stack_trace(last_frame.func_id, insc_ptr);
        insc_ptr = last_frame.ret_addr - 1;

        stack.unwind_shrink_slice();
    }
    exception
}

unsafe fn checked_exception_unwind_stack<A: Alloc>(
    vm: &mut AL31F<A>,
    program: &CompiledProgram<A>,
    checked_exception: CheckedException,
    stack: &mut Stack,
    insc_ptr: usize
) -> Result<(StackSlice, usize), Exception> {
    let exception_type_id: TypeId = (*checked_exception.get_as_dyn_base()).dyn_type_id();

    let mut exception: Exception = Exception::checked_exc(checked_exception);
    let mut insc_ptr: usize = insc_ptr;

    while stack.frames.len() != 0 {
        let frame: &FrameInfo = stack.frames.last().unchecked_unwrap();
        let func_id: usize = frame.func_id;
        exception.push_stack_trace(func_id, insc_ptr);

        let compiled_function: &CompiledFunction = &program.functions[func_id];

        if let Some(exc_handlers /*: &Box<[ExceptionHandlingBlock]>*/)
            = &compiled_function.exc_handlers
        {
            for exc_handler /*: &ExceptionHandlingBlock*/ in exc_handlers.as_ref().iter() {
                let (start_insc, end_insc): (usize, usize) = exc_handler.insc_ptr_range;
                if insc_ptr >= start_insc &&
                    insc_ptr <= end_insc &&
                    exception_type_id == exc_handler.exception_id
                {
                    let frame_size: usize = frame.frame_end - frame.frame_start;
                    let exception_value: Value = Value::new_owned(exception);
                    vm.alloc.add_managed(exception_value.ptr_repr);
                    let mut stack_slice: StackSlice = stack.last_frame_slice();
                    stack_slice.set_value(frame_size - 1, exception_value);

                    return Ok((stack_slice, exc_handler.handler_addr));
                }
            }
        }

        let frame_ret_addr: usize = frame.ret_addr;
        insc_ptr = frame_ret_addr.saturating_sub(1);

        stack.unwind_shrink_slice();
    }

    Err(exception)
}

pub struct VMThreadRunFunctionFut<'a, A: Alloc, const S: bool> {
    thread: &'a mut VMThread<A>,
    slice: StackSlice,
    insc_ptr: usize,

    #[cfg(feature = "async")]
    awaiting_promise: Option<Pin<Box<dyn Future<Output = AsyncReturnType>>>>,

    #[cfg(feature = "async")]
    ret_values_resolver: Option<fn(&mut A, &[Value])>
}

unsafe impl<'a, A: Alloc, const S: bool> Send for VMThreadRunFunctionFut<'a, A, S> {}
unsafe impl<'a, A: Alloc, const S: bool> Sync for VMThreadRunFunctionFut<'a, A, S> {}

unsafe fn poll_unsafe<'a, A: Alloc, const S: bool>(
    this: &mut VMThreadRunFunctionFut<'a, A, S>,
    #[cfg(feature = "async")] cx: &mut Context<'_>,
    #[cfg(not(feature = "async"))] _cx: &mut Context<'_>
) -> Poll<<VMThreadRunFunctionFut<'a, A, S> as Future>::Output> {
    #[cfg(feature = "async")]
    if let Some(awaiting_promise) = &mut this.awaiting_promise {
        if let Poll::Ready(promise_result) = awaiting_promise.poll_unpin(cx) {
            this.awaiting_promise = None;

            let AsyncReturnType(result) = promise_result;
            match result {
                Ok(values) => {
                    if let Some(ret_values_resolver) = this.ret_values_resolver.take() {
                        ret_values_resolver(
                            &mut this.thread.vm.get_shared_data_mut().alloc,
                            &values
                        );
                    }

                    let insc: &Insc = &this.thread.program.as_ref().code[this.insc_ptr - 1];
                    let dests: &Box<[usize]> = if let Insc::Await(_, dests) = insc {
                        dests
                    } else {
                        unreachable_unchecked()
                    };

                    debug_assert_eq!(values.len(), dests.len());
                    for i in 0..dests.len() {
                        let dest: usize = *dests.get_unchecked(i);
                        this.slice.set_value(dest, *values.get_unchecked(i));
                    }
                },
                Err(e) => {
                    match e {
                        Either::Left(checked) => {
                            let (new_slice, insc_ptr_next): (StackSlice, usize) =
                                checked_exception_unwind_stack(
                                    get_vm!(this.thread),
                                    this.thread.program.as_ref(),
                                    checked,
                                    &mut this.thread.stack,
                                    this.insc_ptr
                                )?;
                            this.slice = new_slice;
                            this.insc_ptr = insc_ptr_next;
                        },
                        Either::Right(unchecked) => {
                            return Poll::Ready(Err(unchecked_exception_unwind_stack(
                                unchecked, &mut this.thread.stack, this.insc_ptr
                            )));
                        }
                    }
                }
            }
        } else {
            return Poll::Pending;
        }
    }

    let slice: &mut StackSlice = &mut this.slice;
    let thread: &mut VMThread<A> = &mut this.thread;
    let program: &CompiledProgram<A> = thread.program.as_ref();
    let mut ffi_args: [Value; 32] = [Value::new_null(); 32];
    let mut ffi_rets: [*mut Value; 8] = [std::ptr::null_mut(); 8];

    #[cfg(feature = "async-avoid-block")]
    #[allow(unused)]
    let mut insc_counter: u64 = 0;

    let mut insc_ptr: usize = this.insc_ptr;
    loop {
        #[cfg(feature = "async-avoid-block")]
        if !S {
            insc_counter += 1;
            if insc_counter % 1_000_000 == 0 {
                this.insc_ptr = insc_ptr;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }

        #[cfg(not(debug_assertions))]
        let insc: &Insc = program.code.get_unchecked(insc_ptr);
        #[cfg(debug_assertions)]
        let insc: &Insc = &program.code[insc_ptr];
        insc_ptr += 1;

        match insc {
            Insc::AddInt(src1, src2, dst) =>
                impl_int_binop![slice, src1, src2, dst, wrapping_add],
            Insc::AddFloat(src1, src2, dst) =>
                impl_float_binop![slice, src1, src2, dst, +],
            Insc::AddAny(src1, src2, dst) =>
                impl_checked_op2![slice, src1, src2, dst, checked_add, thread, insc_ptr],
            Insc::IncrInt(pos) => {
                let v: Value = Value::new_int(slice.get_value(*pos).vt_data.inner.int_value + 1);
                slice.set_value(*pos, v);
            },
            Insc::DecrInt(pos) => {
                let v: Value = Value::new_int(slice.get_value(*pos).vt_data.inner.int_value - 1);
                slice.set_value(*pos, v);
            },
            Insc::SubInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, wrapping_sub],
            Insc::SubFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, -],
            Insc::SubAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_sub, thread, insc_ptr],
            Insc::MulInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, wrapping_mul],
            Insc::MulFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, *],
            Insc::MulAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_mul, thread, insc_ptr],
            Insc::DivInt(src1, src2, dst) => {
                let src1: i64 = slice.get_value(*src1).vt_data.inner.int_value;
                let src2: i64 = slice.get_value(*src2).vt_data.inner.int_value;
                if let Some(result) = i64::checked_div(src1, src2) {
                    slice.set_value(*dst, Value::new_int(result))
                } else {
                    return Poll::Ready(Err(unchecked_exception_unwind_stack(
                        UncheckedException::DivideByZero, &mut thread.stack, insc_ptr
                    )));
                }
            },
            Insc::DivFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, /],
            Insc::DivAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_div, thread, insc_ptr],
            Insc::ModInt(src1, src2, dst) => {
                let src1: i64 = slice.get_value(*src1).vt_data.inner.int_value;
                let src2: i64 = slice.get_value(*src2).vt_data.inner.int_value;
                if let Some(result) = i64::checked_rem(src1, src2) {
                    slice.set_value(*dst, Value::new_int(result))
                } else {
                    return Poll::Ready(Err(unchecked_exception_unwind_stack(
                        UncheckedException::DivideByZero, &mut thread.stack, insc_ptr
                    )));
                }
            },
            Insc::ModAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_mod, thread, insc_ptr],
            Insc::EqValue(src1, src2, dst) => {
                debug_assert_eq!(slice.get_value(*src1).vt_data.tag,
                                 slice.get_value(*src2).vt_data.tag);
                let src1: u64 = slice.get_value(*src1).vt_data.inner.repr;
                let src2: u64 = slice.get_value(*src2).vt_data.inner.repr;
                slice.set_value(*dst, Value::new_bool(src1 == src2));
            },
            Insc::EqRef(src1, src2, dst) => {
                let src1: usize = slice.get_value(*src1).ptr_repr.ptr;
                let src2: usize = slice.get_value(*src2).ptr_repr.ptr;
                slice.set_value(*dst, Value::new_bool(src1 == src2));
            },
            Insc::EqAny(src1, src2, dst) => {
                let src1: WidePointer = slice.get_value(*src1).ptr_repr;
                let src2: WidePointer = slice.get_value(*src2).ptr_repr;
                slice.set_value(*dst, Value::new_bool(src1 == src2));
            },
            Insc::NeValue(src1, src2, dst) => {
                debug_assert_eq!(slice.get_value(*src1).vt_data.tag,
                                 slice.get_value(*src2).vt_data.tag);
                let src1: u64 = slice.get_value(*src1).vt_data.inner.repr;
                let src2: u64 = slice.get_value(*src2).vt_data.inner.repr;
                slice.set_value(*dst, Value::new_bool(src1 != src2));
            },
            Insc::NeRef(src1, src2, dst) => {
                let src1: usize = slice.get_value(*src1).ptr_repr.ptr;
                let src2: usize = slice.get_value(*src2).ptr_repr.ptr;
                slice.set_value(*dst, Value::new_bool(src1 != src2));
            },
            Insc::NeAny(src1, src2, dst) => {
                let src1: WidePointer = slice.get_value(*src1).ptr_repr;
                let src2: WidePointer = slice.get_value(*src2).ptr_repr;
                slice.set_value(*dst, Value::new_bool(src1 != src2));
            },
            Insc::LtInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <, i64, int_value],
            Insc::LtFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <, f64, float_value],
            Insc::LtAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_lt, thread, insc_ptr],
            Insc::GtInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >, i64, int_value],
            Insc::GtFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >, f64, float_value],
            Insc::GtAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_gt, thread, insc_ptr],
            Insc::LeInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <=, i64, int_value],
            Insc::LeFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <=, f64, float_value],
            Insc::LeAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_le, thread, insc_ptr],
            Insc::GeInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >=, i64, int_value],
            Insc::GeFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >=, f64, float_value],
            Insc::GeAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_ge, thread, insc_ptr],
            Insc::BAndInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, &],
            Insc::BAndAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_bit_and, thread, insc_ptr],
            Insc::BOrInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, |],
            Insc::BOrAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_bit_or, thread, insc_ptr],
            Insc::BXorInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, ^],
            Insc::BXorAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_bit_xor, thread, insc_ptr],
            Insc::BNotInt(src, dst) => {
                let src: u64 = slice.get_value(*src).vt_data.inner.repr;
                slice.set_value(*dst, Value::new_raw_value(INT_TYPE_TAG, u64::reverse_bits(src)));
            },
            Insc::BNotAny(src, dst) =>
                impl_checked_unary_op![slice, src, dst, checked_bit_not, thread, insc_ptr],
            Insc::NegInt(src, dst) => {
                let src: i64 = slice.get_value(*src).vt_data.inner.int_value;
                slice.set_value(*dst, Value::new_int(i64::wrapping_neg(src)));
            },
            Insc::NegFloat(src, dst) => {
                let src: f64 = slice.get_value(*src).vt_data.inner.float_value;
                slice.set_value(*dst, Value::new_float(-src));
            },
            Insc::NegAny(src, dst) =>
                impl_checked_unary_op![slice, src, dst, checked_neg, thread, insc_ptr],
            Insc::AndBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, &],
            Insc::AndAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_logic_and, thread, insc_ptr],
            Insc::OrBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, |],
            Insc::OrAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_logic_or, thread, insc_ptr],
            Insc::NotBool(src, dst) => {
                let src: bool = slice.get_value(*src).vt_data.inner.bool_value;
                slice.set_value(*dst, Value::new_bool(!src));
            },
            Insc::NotAny(src, dst) =>
                impl_checked_unary_op![slice, src, dst, checked_not, thread, insc_ptr],
            Insc::ShlInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, <<],
            Insc::ShlAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_bit_shl, thread, insc_ptr],
            Insc::ShrInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, >>],
            Insc::ShrAny(src1, src2, dst) =>
                impl_checked_bin_op![slice, src1, src2, dst, checked_bit_shr, thread, insc_ptr],
            Insc::MakeIntConst(i64_const, dst) =>
                slice.set_value(*dst, Value::new_int(*i64_const)),
            Insc::MakeFloatConst(f64_const, dst) =>
                slice.set_value(*dst, Value::new_float(*f64_const)),
            Insc::MakeCharConst(char_const, dst) =>
                slice.set_value(*dst, Value::new_char(*char_const)),
            Insc::MakeBoolConst(bool_const, dst) =>
                slice.set_value(*dst, Value::new_bool(*bool_const)),
            Insc::MakeNull(dst) =>
                slice.set_value(*dst, Value::new_null()),
            Insc::LoadConst(const_id, dst) => {
                let constant: Value = *thread.program.as_ref().const_pool.get_unchecked(*const_id);
                slice.set_value(*dst, constant);
            }
            Insc::SaveConst(const_src, const_id) => {
                let constant: Value = slice.get_value(*const_src);
                *thread.program.as_mut().const_pool.get_unchecked_mut(*const_id) = constant;
            }
            Insc::CastFloatInt(src, dst) =>
                impl_cast_op![slice, src, dst, f64, i64, float_value, new_int],
            Insc::CastBoolInt(src, dst) =>
                impl_cast_op![slice, src, dst, bool, i64, bool_value, new_int],
            Insc::CastAnyInt(src, dst) =>
                impl_checked_cast_op![slice, src, dst, cast_any_int, thread, insc_ptr],
            Insc::CastIntFloat(src, dst) =>
                impl_cast_op![slice, src, dst, i64, f64, int_value, new_float],
            Insc::CastAnyFloat(src, dst) =>
                impl_checked_cast_op![slice, src, dst, cast_any_float, thread, insc_ptr],
            Insc::CastAnyChar(src, dst) =>
                impl_checked_cast_op![slice, src, dst, cast_any_char, thread, insc_ptr],
            Insc::CastIntBool(src, dst) => {
                let src: i64 = slice.get_value(*src).vt_data.inner.int_value;
                let casted: bool = src != 0;
                slice.set_value(*dst, Value::new_bool(casted));
            }
            Insc::CastAnyBool(src, dst) =>
                impl_checked_cast_op![slice, src, dst, cast_any_bool, thread, insc_ptr],
            Insc::IsNull(src, dst) => {
                let src: Value = slice.get_value(*src);
                slice.set_value(*dst, Value::new_bool(src.is_null()));
            },
            Insc::NullCheck(src) => {
                let src: Value = slice.get_value(*src);
                if src.is_null() {
                    return Poll::Ready(Err(unchecked_exception_unwind_stack(
                        UncheckedException::UnexpectedNull { value: src },
                        &mut thread.stack,
                        insc_ptr
                    )));
                }
            },
            Insc::TypeCheck(src, _tyck_info) => {
                let _src: Value = slice.get_value(*src);
                todo!();
            },
            Insc::Call(func_id, args, rets) => {
                #[cfg(not(debug_assertions))]
                    let compiled: &CompiledFunction = program.functions.get_unchecked(*func_id);
                #[cfg(debug_assertions)]
                    let compiled: &CompiledFunction = &program.functions[*func_id];

                debug_assert_eq!(compiled.arg_count, args.len());
                *slice = thread.stack.func_call_grow_stack(
                    *func_id,
                    compiled.stack_size,
                    args,
                    NonNull::from(&rets[..]),
                    insc_ptr
                );
                insc_ptr = compiled.start_addr;
            },
            Insc::CallPtr(func_id_loc, args, rets) => {
                let func_id: usize = slice.get_value(*func_id_loc).vt_data.inner.int_value as usize;

                #[cfg(not(debug_assertions))]
                    let compiled: &CompiledFunction = program.functions.get_unchecked(func_id);
                #[cfg(debug_assertions)]
                    let compiled: &CompiledFunction = &program.functions[func_id];

                debug_assert_eq!(compiled.arg_count, args.len());
                *slice = thread.stack.func_call_grow_stack(
                    func_id,
                    compiled.stack_size,
                    args,
                    NonNull::from(&rets[..]),
                    insc_ptr
                );
                insc_ptr = compiled.start_addr;
            },
            Insc::CallOverload(_, _, _) => {}
            Insc::ReturnNothing => {
                if let Some((prev_stack_slice, ret_addr)) =
                    thread.stack.done_func_call_shrink_stack0()
                {
                    insc_ptr = ret_addr;
                    *slice = prev_stack_slice;
                } else {
                    return Poll::Ready(Ok(vec![]));
                }
            },
            Insc::ReturnOne(ret_value) => {
                if let Some((prev_stack_slice, ret_addr)) =
                    thread.stack.done_func_call_shrink_stack1(*ret_value)
                {
                    insc_ptr = ret_addr;
                    *slice = prev_stack_slice;
                } else {
                    return Poll::Ready(Ok(vec![slice.get_value(*ret_value)]));
                }
            },
            Insc::Return(ret_values) => {
                if let Some((prev_stack_slice, ret_addr)) =
                    thread.stack.done_func_call_shrink_stack(&ret_values)
                {
                    insc_ptr = ret_addr;
                    *slice = prev_stack_slice;
                } else {
                    let mut ret_vec: Vec<Value> = Vec::with_capacity(ret_values.len());
                    for ret_value_loc in ret_values.iter() {
                        ret_vec.push(slice.get_value(*ret_value_loc));
                    }
                    return Poll::Ready(Ok(ret_vec));
                }
            },
            #[cfg(feature = "optimized-rtlc")]
            Insc::FFICallRtlc(ffi_func_id, args, ret_value_locs) => {
                let ffi_function: &Box<dyn FFIFunction<Combustor<A>>>
                    = &program.ffi_funcs[*ffi_func_id];

                let args_len: usize = args.len();
                for i /*: usize*/ in 0..args_len {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }

                let ret_locs_len: usize = ret_value_locs.len();
                for i /*: usize*/ in 0..ret_locs_len {
                    let ret_value_loc_idx: usize = *ret_value_locs.get_unchecked(i);
                    *ffi_rets.get_unchecked_mut(i) = slice.get_value_mut_ref(ret_value_loc_idx);
                }

                let mut combustor: Combustor<A> = Combustor::new(NonNull::from(get_vm!(thread)));

                if let Err(e /*: FFIException*/) = ffi_function.call_rtlc(
                    &mut combustor,
                    &ffi_args[0..args_len],
                    &mut ffi_rets[0..ret_locs_len]
                ) {
                    match e {
                        Either::Left(checked) => {
                            let (new_slice, insc_ptr_next): (StackSlice, usize) =
                                checked_exception_unwind_stack(
                                    get_vm!(thread),
                                    &program,
                                    checked,
                                    &mut thread.stack,
                                    insc_ptr
                                )?;
                            *slice = new_slice;
                            insc_ptr = insc_ptr_next;
                        },
                        Either::Right(unchecked) => {
                            return Poll::Ready(Err(unchecked_exception_unwind_stack(
                                unchecked, &mut thread.stack, insc_ptr
                            )));
                        }
                    }
                }
            },
            Insc::FFICall(ffi_func_id, args, ret_value_locs) => {
                let ffi_function: &Box<dyn FFIFunction<Combustor<A>>>
                    = &program.ffi_funcs[*ffi_func_id];

                let args_len: usize = args.len();
                for i /*: usize*/ in 0..args_len {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }

                let ret_locs_len: usize = ret_value_locs.len();
                for i /*: usize*/ in 0..ret_locs_len {
                    let ret_value_loc_idx: usize = *ret_value_locs.get_unchecked(i);
                    *ffi_rets.get_unchecked_mut(i) = slice.get_value_mut_ref(ret_value_loc_idx);
                }

                let mut combustor: Combustor<A> = Combustor::new(NonNull::from(get_vm!(thread)));

                if let Err(e /*: FFIException*/) = ffi_function.call_unchecked(
                    &mut combustor,
                    &ffi_args[0..args_len],
                    &mut ffi_rets[0..ret_locs_len]
                ) {
                    match e {
                        Either::Left(checked) => {
                            let (new_slice, insc_ptr_next): (StackSlice, usize) =
                                checked_exception_unwind_stack(
                                    get_vm!(thread),
                                    &program,
                                    checked,
                                    &mut thread.stack,
                                    insc_ptr
                                )?;
                            *slice = new_slice;
                            insc_ptr = insc_ptr_next;
                        },
                        Either::Right(unchecked) => {
                            return Poll::Ready(Err(unchecked_exception_unwind_stack(
                                unchecked, &mut thread.stack, insc_ptr
                            )));
                        }
                    }
                }
            },
            #[cfg(all(feature = "optimized-rtlc", feature = "async"))]
            Insc::FFICallAsync(async_ffi_func_id, args, ret) => {
                let async_ffi_function: &Box<dyn FFIAsyncFunction<A, AsyncCombustor<A>>>
                    = &program.async_ffi_funcs[*async_ffi_func_id];

                let args_len: usize = args.len();
                for i /*: usize*/ in 0..args_len {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }

                let mut combustor: AsyncCombustor<A> =
                    AsyncCombustor::new(thread.vm.serializer.clone());

                match async_ffi_function.call_rtlc(&mut combustor, &ffi_args[0..args_len]) {
                    Ok(promise /*: Promise*/) => {
                        let promise: Value = Value::new_owned(promise);
                        thread.vm.get_shared_data_mut().alloc.add_managed(promise.ptr_repr);
                        slice.set_value(*ret, promise);
                    },
                    Err(e /*: FFIException*/) => {
                        match e {
                            Either::Left(checked) => {
                                let (new_slice, insc_ptr_next): (StackSlice, usize) =
                                    checked_exception_unwind_stack(
                                        get_vm!(thread),
                                        &program,
                                        checked,
                                        &mut thread.stack,
                                        insc_ptr
                                    )?;
                                *slice = new_slice;
                                insc_ptr = insc_ptr_next;
                            },
                            Either::Right(unchecked) => {
                                return Poll::Ready(Err(unchecked_exception_unwind_stack(
                                    unchecked, &mut thread.stack, insc_ptr
                                )));
                            }
                        }
                    }
                }
            },
            #[cfg(feature = "async")]
            Insc::Await(promise, _) => {
                let promise: Value = slice.get_value(*promise);
                let wrapper: *mut Wrapper<()> = promise.ptr_repr.ptr as *mut Wrapper<()>;
                if (*wrapper).ownership_info == OwnershipInfo::MovedToRust as u8 {
                    return Poll::Ready(Err(unchecked_exception_unwind_stack(
                        UncheckedException::AlreadyAwaited { promise },
                        &mut thread.stack,
                        insc_ptr
                    )));
                }

                let promise: Promise<A> = promise.move_out::<Promise<A>>();
                (*wrapper).ownership_info = OwnershipInfo::MovedToRust as u8;

                this.insc_ptr = insc_ptr;

                let thread: &'static VMThread<A> = transmute::<_, _>(thread);
                this.ret_values_resolver = promise.ret_values_resolver;
                this.awaiting_promise = Some(Box::pin(thread.vm.co_await(promise)));
                cx.waker().wake_by_ref();
                return Poll::Pending;
            },
            Insc::Raise(exception_ptr) => {
                let exception: Value = slice.get_value(*exception_ptr);
                let (new_slice, insc_ptr_next): (StackSlice, usize) =
                    checked_exception_unwind_stack(
                        get_vm!(thread),
                        &program,
                        exception,
                        &mut thread.stack,
                        insc_ptr
                    )?;
                *slice = new_slice;
                insc_ptr = insc_ptr_next;
                continue;
            },
            Insc::JumpIfTrue(condition, dest) => {
                let condition: bool = slice.get_value(*condition).vt_data.inner.bool_value;
                if condition {
                    insc_ptr = *dest;
                }
            },
            Insc::JumpIfFalse(condition, dest) => {
                let condition: bool = slice.get_value(*condition).vt_data.inner.bool_value;
                if !condition {
                    insc_ptr = *dest;
                }
            },
            Insc::Jump(dest) => {
                insc_ptr = *dest;
            },
            Insc::CreateString(dest) => {
                let string: String = String::new();
                let string: Value = Value::new_owned(string);
                get_vm!(thread).alloc.add_managed(string.ptr_repr);
                slice.set_value(*dest, string);
            },
            Insc::CreateObject(dest) => {
                let object: Object = Object::new();
                let object: Value = Value::new_owned(object);
                get_vm!(thread).alloc.add_managed(object.ptr_repr);
                slice.set_value(*dest, object);
            },
            Insc::CreateContainer(ctor, vt, dest) => {
                let container: Value = Value::new_container(ctor(), vt.as_ref());
                get_vm!(thread).alloc.add_managed(container.ptr_repr);
                slice.set_value(*dest, container);
            },
            Insc::VecIndex(_, _, _) => {}
            Insc::VecIndexPut(_, _, _) => {}
            Insc::VecInsert(_, _, _) => {}
            Insc::VecRemove(_, _, _) => {}
            Insc::VecLen(_) => {}
            Insc::VecClear(_) => {}

            Insc::DenseVecIndex(_, _, _) => {}
            Insc::DenseVecIndexPut(_, _, _) => {}
            Insc::DenseVecInsert(_, _, _) => {}
            Insc::DenseVecRemove(_, _, _) => {}
            Insc::DenseVecLen(_) => {}
            Insc::DenseVecClear(_) => {}

            Insc::StrConcat(src1, src2, dest) => {
                let src1: &String = &*(slice.get_value(*src1).get_as_mut_ptr_norm() as *const _);
                let src2: &String = &*(slice.get_value(*src2).get_as_mut_ptr_norm() as *const _);
                let mut buffer: String = (*src1).clone();
                buffer.push_str(&*src2);

                let dest_value: Value = Value::new_owned(buffer);
                get_vm!(thread).alloc.add_managed(dest_value.ptr_repr);
                slice.set_value(*dest, dest_value);
            },
            Insc::StrFormat(_, _, _) => {}
            Insc::StrLen(_, _) => {}
            Insc::StrSlice(_, _, _, _) => {}
            Insc::StrEquals(_, _) => {}

            Insc::ObjectGet(src, field, dest) => {
                let object: &Object = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                let value: Value = *object.fields.get(field.as_ref()).unwrap_or(&Value::new_null());
                slice.set_value(*dest, value);
            },
            Insc::ObjectGetDyn(src, field, dest) => {
                let object: &Object = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                let field: &String = &*(slice.get_value(*field).get_as_mut_ptr_norm() as *const _);
                let value: Value = *object.fields.get(field).unwrap_or(&Value::new_null());
                slice.set_value(*dest, value);
            },
            Insc::ObjectPut(src, field, data) => {
                let object: &mut Object = &mut *(slice.get_value(*src).get_as_mut_ptr_norm());
                let data: Value = slice.get_value(*data);
                object.fields.insert(field.as_ref().to_string(), data);
            },
            Insc::ObjectPutDyn(src, field, data) => {
                let object: &mut Object = &mut *(slice.get_value(*src).get_as_mut_ptr_norm());
                let field: &String = &*(slice.get_value(*field).get_as_mut_ptr_norm() as *const _);
                let data: Value = slice.get_value(*data);
                object.fields.insert(field.to_string(), data);
            }
        }
    }
}

impl<'a, A: Alloc, const S: bool> Future for VMThreadRunFunctionFut<'a, A, S> {
    type Output = Result<Vec<Value>, Exception>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { poll_unsafe(Pin::into_inner(self), cx) }
    }
}

pub unsafe fn vm_thread_run_function<'a, A: Alloc, const S: bool>(
    arg_pack: UncheckedSendSync<(&'a mut VMThread<A>, usize, &[Value])>
) -> Result<VMThreadRunFunctionFut<'a, A, S>, Exception> {
    let (thread, func_id, args) = arg_pack.into_inner();

    get_vm!(thread).alloc.set_gc_allowed(true);

    let program: &CompiledProgram<A> = thread.program.as_ref();
    let compiled_function: &CompiledFunction = &program.functions[func_id];
    if compiled_function.arg_count != args.len() {
        let exception: UncheckedException = UncheckedException::ArgCountMismatch {
            func_id, expected: compiled_function.arg_count, got: args.len()
        };
        return Err(Exception::unchecked_exc(exception));
    }

    let slice: StackSlice =
        thread.stack.ext_func_call_grow_stack(func_id, compiled_function.stack_size, args);
    let insc_ptr: usize = compiled_function.start_addr;

    Ok(VMThreadRunFunctionFut {
        thread,
        slice,
        insc_ptr,

        #[cfg(feature = "async")] awaiting_promise: None,
        #[cfg(feature = "async")] ret_values_resolver: None
    })
}
