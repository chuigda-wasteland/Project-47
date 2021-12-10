use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::marker::PhantomPinned;
use std::task::{Context, Poll};

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::unchecked::{UncheckedCellOps, UncheckedSendSync};
use xjbutil::wide_ptr::WidePointer;

use crate::builtins::closure::Closure;
use crate::builtins::object::Object;
use crate::builtins::vec::VMGenericVec;
use crate::data::Value;
use crate::data::exception::{Exception, UncheckedException};
use crate::data::value_typed::INT_TYPE_TAG;
use crate::ffi::FFIException;
use crate::ffi::sync_fn::Function as FFIFunction;
use crate::vm::al31f::{AL31F, Combustor};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::executor::checked_bin_ops::*;
use crate::vm::al31f::executor::checked_cast_ops::*;
use crate::vm::al31f::executor::checked_unary_ops::*;
use crate::vm::al31f::executor::overload::call_overload;
use crate::vm::al31f::executor::rtti::check_type;
use crate::vm::al31f::executor::unwinding::*;
use crate::vm::al31f::insc::Insc;
use crate::vm::al31f::stack::{Stack, StackSlice};

#[cfg(feature = "async")] use std::hint::unreachable_unchecked;
#[cfg(feature = "async")] use std::mem::transmute;
#[cfg(feature = "async")] use futures::FutureExt;
#[cfg(feature = "async")] use crate::data::wrapper::{Wrapper, OwnershipInfo};
#[cfg(feature = "async")] use crate::ffi::async_fn::{AsyncReturnType, Promise};
#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncFunction as FFIAsyncFunction;
#[cfg(feature = "async")] use crate::ffi::async_fn::PromiseContext;
#[cfg(feature = "async")] use crate::util::serializer::CoroutineContext;
#[cfg(feature = "async")] use crate::vm::al31f::AsyncCombustor;

#[cfg(all(feature = "async", feature = "al31f-builtin-ops"))]
use crate::vm::al31f::executor::coroutine_spawn::coroutine_spawn;

include!("get_vm_makro.rs");
include!("impl_makro.rs");

pub struct VMThread<A: Alloc> {
    #[cfg(feature = "async")]
    pub vm: CoroutineContext<AL31F<A>>,
    #[cfg(not(feature = "async"))]
    pub vm: AL31F<A>,

    pub program: NonNull<CompiledProgram<A>>,
    pub stack: Stack,

    pub _phantom: PhantomPinned
}

impl<A: Alloc> Drop for VMThread<A> {
    fn drop(&mut self) {
        unsafe {
            self.vm.get_shared_data_mut().alloc.remove_stack(&self.stack);
        }
    }
}

unsafe impl<A: Alloc> Send for VMThread<A> {}
unsafe impl<A: Alloc> Sync for VMThread<A> {}

#[must_use = "VM threads are effective iff a function gets run on it"]
#[cfg(feature = "async")]
pub async fn create_vm_main_thread<A: Alloc>(
    alloc: A,
    program: &CompiledProgram<A>
) -> Box<VMThread<A>> {
    let mut ret = Box::new(VMThread {
        vm: CoroutineContext::main_context(AL31F::new(alloc)).await,
        program: NonNull::from(program),
        stack: Stack::new(),
        _phantom: PhantomPinned::default()
    });
    unsafe { ret.vm.get_shared_data_mut().alloc.add_stack(&ret.stack) };
    ret
}

#[must_use = "VM threads are effective iff a function gets run on it"]
#[cfg(feature = "async")]
pub fn create_vm_child_thread<A: Alloc>(
    child_context: CoroutineContext<AL31F<A>>,
    program: NonNull<CompiledProgram<A>>
) -> Box<VMThread<A>> {
    let mut ret = Box::new(VMThread {
        vm: child_context,
        program,
        stack: Stack::new(),
        _phantom: PhantomPinned::default()
    });
    unsafe { ret.vm.get_shared_data_mut().alloc.add_stack(&ret.stack) };
    ret
}

pub struct VMThreadRunFunctionFut<'a, A: Alloc, const S: bool> {
    thread: &'a mut VMThread<A>,
    slice: StackSlice,
    insc_ptr: usize,

    #[cfg(feature = "async")]
    awaiting_promise: Option<(Pin<Box<dyn Future<Output = AsyncReturnType>>>, PromiseContext<A>)>,
}

unsafe impl<'a, A: Alloc, const S: bool> Send for VMThreadRunFunctionFut<'a, A, S> {}
unsafe impl<'a, A: Alloc, const S: bool> Sync for VMThreadRunFunctionFut<'a, A, S> {}

unsafe fn poll_unsafe<'a, A: Alloc, const S: bool>(
    this: &mut VMThreadRunFunctionFut<'a, A, S>,
    #[cfg(feature = "async")] cx: &mut Context<'_>,
    #[cfg(not(feature = "async"))] _cx: &mut Context<'_>
) -> Poll<Result<Vec<Value>, Exception>> {
    #[cfg(feature = "async")]
    if let Some((fut, _)) = &mut this.awaiting_promise {
        if let Poll::Ready(promise_result) = fut.poll_unpin(cx) {
            let (_, mut ctx) = this.awaiting_promise.take().unchecked_unwrap();

            let AsyncReturnType(result) = promise_result;
            match result {
                Ok(values) => {
                    if let Some(resolver) = ctx.resolver.take() {
                        resolver(
                            &mut this.thread.vm.get_shared_data_mut().alloc,
                            &values
                        );
                    }

                    let insc: &Insc = &this.thread.program.as_ref().code[this.insc_ptr - 1];
                    let dests: &[usize] = if let Insc::Await(_, dests) = insc {
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
                        FFIException::Checked(checked) => {
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
                        FFIException::Unchecked(unchecked) => {
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
            if insc_counter == 500_000 {
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
            Insc::IsType(src, tyck_info, dest) => {
                let src: Value = slice.get_value(*src);
                slice.set_value(*dest, Value::new_bool(check_type(src, *tyck_info)));
            },
            Insc::TypeCheck(src, tyck_info) => {
                let src: Value = slice.get_value(*src);
                if !check_type(src, *tyck_info) {
                    return Poll::Ready(Err(unchecked_exception_unwind_stack(
                        UncheckedException::TypeCheckFailure {
                            object: src,
                            expected_type: *tyck_info
                        },
                        &mut thread.stack,
                        insc_ptr
                    )));
                }
            },
            Insc::OwnershipInfoCheck(src, mask) => {
                let src: Value = slice.get_value(*src);
                if src.is_value() || src.ownership_info_norm() as u8 & mask != 0 {
                    return Poll::Ready(Err(unchecked_exception_unwind_stack(
                        UncheckedException::OwnershipCheckFailure {
                            object: src,
                            expected_mask: *mask
                        },
                        &mut thread.stack,
                        insc_ptr
                    )));
                }
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
            Insc::CallPtr(func, args, rets) => {
                let func: Value = slice.get_value(*func);
                if func.is_value() {
                    let func_id: usize = func.vt_data.inner.int_value as usize;

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
                } else {
                    let closure: &Closure = &*(func.get_as_mut_ptr_norm::<Closure>() as *const _);
                    let func_id: usize = closure.func_id;

                    #[cfg(not(debug_assertions))]
                    let compiled: &CompiledFunction = program.functions.get_unchecked(func_id);
                    #[cfg(debug_assertions)]
                    let compiled: &CompiledFunction = &program.functions[func_id];

                    *slice = thread.stack.closure_call_grow_stack(
                        func_id,
                        compiled.stack_size,
                        &closure.captures,
                        args,
                        NonNull::from(&rets[..]),
                        insc_ptr
                    );
                    insc_ptr = compiled.start_addr;
                }
            },
            Insc::CallOverload(overload_table, args, rets) => {
                match call_overload(
                    thread,
                    *slice,
                    insc_ptr,
                    *overload_table,
                    args,
                    rets)
                {
                    Ok((new_slice, new_insc_ptr)) => {
                        *slice = new_slice;
                        insc_ptr = new_insc_ptr;
                    },
                    Err(err) => {
                        return Poll::Ready(Err(err));
                    }
                }
            },
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
            Insc::FFICallRtlc(ffi_func_id, args, ret_value_locs) => {
                let ffi_function: &'static dyn FFIFunction<Combustor<A>>
                    = program.ffi_funcs[*ffi_func_id];

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
                        FFIException::Checked(checked) => {
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
                        FFIException::Unchecked(unchecked) => {
                            return Poll::Ready(Err(unchecked_exception_unwind_stack(
                                unchecked, &mut thread.stack, insc_ptr
                            )));
                        }
                    }
                }
            },
            #[cfg(feature = "optimized-rtlc")]
            Insc::FFICall(ffi_func_id, args, ret_value_locs) => {
                #[cfg(not(debug_assertions))]
                let ffi_function: &'static dyn FFIFunction<Combustor<A>>
                    = *program.ffi_funcs.get_unchecked(*ffi_func_id);
                #[cfg(debug_assertions)]
                let ffi_function: &'static dyn FFIFunction<Combustor<A>>
                    = program.ffi_funcs[*ffi_func_id];

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
                        FFIException::Checked(checked) => {
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
                        FFIException::Unchecked(unchecked) => {
                            return Poll::Ready(Err(unchecked_exception_unwind_stack(
                                unchecked, &mut thread.stack, insc_ptr
                            )));
                        }
                    }
                }
            },
            #[cfg(all(feature = "optimized-rtlc", feature = "async"))]
            Insc::FFICallAsync(async_ffi_func_id, args, ret) => {
                #[cfg(not(debug_assertions))]
                let async_ffi_function: &'static dyn FFIAsyncFunction<A, _, _>
                    = *program.async_ffi_funcs.get_unchecked(*async_ffi_func_id);
                #[cfg(debug_assertions)]
                let async_ffi_function: &'static dyn FFIAsyncFunction<A, _, _>
                    = program.async_ffi_funcs[*async_ffi_func_id];

                let args_len: usize = args.len();
                for i /*: usize*/ in 0..args_len {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }

                let mut combustor: AsyncCombustor<A> = AsyncCombustor::new(
                    thread.vm.serializer.clone(),
                    thread.program
                );

                match async_ffi_function.call_rtlc(&mut combustor, &ffi_args[0..args_len]) {
                    Ok(promise /*: Promise*/) => {
                        let promise: Value = Value::new_owned(promise);
                        thread.vm.get_shared_data_mut().alloc.add_managed(promise.ptr_repr);
                        slice.set_value(*ret, promise);
                    },
                    Err(e /*: FFIException*/) => {
                        match e {
                            FFIException::Checked(checked) => {
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
                            FFIException::Unchecked(unchecked) => {
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

                let Promise { fut, ctx } = promise.move_out::<Promise<A>>();
                (*wrapper).ownership_info = OwnershipInfo::MovedToRust as u8;

                this.insc_ptr = insc_ptr;

                let thread: &'static VMThread<A> = transmute::<_, _>(thread);
                this.awaiting_promise = Some((Box::pin(thread.vm.co_await(fut)), ctx));
                cx.waker().wake_by_ref();
                return Poll::Pending;
            },
            #[cfg(all(feature = "async", feature = "al31f-builtin-ops"))]
            Insc::Spawn(func, args) => {
                let promise: Promise<A> = coroutine_spawn(thread, slice, *func, args);
                this.awaiting_promise = Some((promise.fut, promise.ctx));
                this.insc_ptr = insc_ptr + 1;
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
            Insc::CreateContainer(ctor, vt, dest) => {
                let container: Value = Value::new_container(ctor(), vt.as_ref());
                get_vm!(thread).alloc.add_managed(container.ptr_repr);
                slice.set_value(*dest, container);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::CreateString(dest) => {
                let string: String = String::new();
                let string: Value = Value::new_owned(string);
                get_vm!(thread).alloc.add_managed(string.ptr_repr);
                slice.set_value(*dest, string);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::CreateObject(dest) => {
                let object: Object = Object::new();
                let object: Value = Value::new_owned(object);
                get_vm!(thread).alloc.add_managed(object.ptr_repr);
                slice.set_value(*dest, object);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::VecIndex(src, index, dst) => {
                let vec_value: Value = slice.get_value(*src);
                let vec: &VMGenericVec = &*(vec_value.get_as_mut_ptr_norm() as *const _);
                let index: i64 = slice.get_value(*index).vt_data.inner.int_value;
                if let Some(data) = vec.inner.get_ref_unchecked().get(index as usize) {
                    slice.set_value(*dst, *data);
                } else {
                    return Poll::Ready(Err(
                        unchecked_exception_unwind_stack(
                            UncheckedException::IndexOutOfBounds { indexed: vec_value, index },
                            &mut thread.stack,
                            insc_ptr
                        )
                    ));
                }
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::VecIndexPut(src, index, value) => {
                let vec_value: Value = slice.get_value(*src);
                let vec: &VMGenericVec = &*(vec_value.get_as_mut_ptr_norm() as *const _);
                let index: i64 = slice.get_value(*index).vt_data.inner.int_value;
                if let Some(data) = vec.inner.get_mut_ref_unchecked().get_mut(index as usize) {
                    let value: Value = slice.get_value(*value);
                    get_vm!(thread).alloc.mark_object(value.ptr_repr);
                    *data = value;
                } else {
                    return Poll::Ready(Err(
                        unchecked_exception_unwind_stack(
                            UncheckedException::IndexOutOfBounds { indexed: vec_value, index },
                            &mut thread.stack,
                            insc_ptr
                        )
                    ));
                }
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::VecPush(src, data) => {
                let vec_value: Value = slice.get_value(*src);
                let vec: &VMGenericVec = &*(vec_value.get_as_mut_ptr_norm() as *const _);
                let data: Value = slice.get_value(*data);
                get_vm!(thread).alloc.mark_object(data.ptr_repr);
                vec.inner.get_mut_ref_unchecked().push(data);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::VecLen(src, dst) => {
                let vec_value: Value = slice.get_value(*src);
                let vec: &VMGenericVec = &*(vec_value.get_as_mut_ptr_norm() as *const _);
                slice.set_value(*dst, Value::new_int(vec.inner.get_ref_unchecked().len() as i64));
            },

            #[cfg(feature = "al31f-builtin-ops")]
            Insc::StrClone(src, dest) => {
                let src: &String = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                let buffer: String = src.clone();

                let dest_value: Value = Value::new_owned(buffer);
                get_vm!(thread).alloc.add_managed(dest_value.ptr_repr);
                slice.set_value(*dest, dest_value);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::StrConcat(sources, dest) => {
                let mut buffer: String = String::new();
                for src in sources.into_iter() {
                    let src: &String = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                    buffer.push_str(src);
                }

                let dest_value: Value = Value::new_owned(buffer);
                get_vm!(thread).alloc.add_managed(dest_value.ptr_repr);
                slice.set_value(*dest, dest_value);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::StrLen(src, dest) => {
                let src: &String = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                slice.set_value(*dest, Value::new_int(src.len() as i64));
            }
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::StrEquals(src1, src2, dst) => {
                let src1: &String = &*(slice.get_value(*src1).get_as_mut_ptr_norm() as *const _);
                let src2: &String = &*(slice.get_value(*src2).get_as_mut_ptr_norm() as *const _);
                slice.set_value(*dst, Value::new_bool(src1 == src2));
            }

            #[cfg(feature = "al31f-builtin-ops")]
            Insc::ObjectGet(src, field, dest) => {
                let object: &Object = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                let value: Value = *object.fields
                    .get_ref_unchecked()
                    .get(field.as_ref())
                    .unwrap_or(&Value::new_null());
                slice.set_value(*dest, value);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::ObjectGetDyn(src, field, dest) => {
                let object: &Object = &*(slice.get_value(*src).get_as_mut_ptr_norm() as *const _);
                let field: &String = &*(slice.get_value(*field).get_as_mut_ptr_norm() as *const _);
                let value: Value = *object.fields
                    .get_ref_unchecked()
                    .get(field)
                    .unwrap_or(&Value::new_null());
                slice.set_value(*dest, value);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::ObjectPut(src, field, data) => {
                let object: &mut Object = &mut *(slice.get_value(*src).get_as_mut_ptr_norm());
                let data: Value = slice.get_value(*data);
                get_vm!(thread).alloc.mark_object(data.ptr_repr);
                object.fields.get_mut_ref_unchecked().insert(field.as_ref().to_string(), data);
            },
            #[cfg(feature = "al31f-builtin-ops")]
            Insc::ObjectPutDyn(src, field, data) => {
                let object: &mut Object = &mut *(slice.get_value(*src).get_as_mut_ptr_norm());
                let field: &String = &*(slice.get_value(*field).get_as_mut_ptr_norm() as *const _);
                let data: Value = slice.get_value(*data);
                get_vm!(thread).alloc.mark_object(data.ptr_repr);
                object.fields.get_mut_ref_unchecked().insert(field.to_string(), data);
            }
        }
    }
}

impl<'a, A: Alloc, const S: bool> Future for VMThreadRunFunctionFut<'a, A, S> {
    type Output = UncheckedSendSync<Result<Vec<Value>, Exception>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match poll_unsafe(Pin::into_inner(self), cx) {
                Poll::Ready(r) => Poll::Ready(UncheckedSendSync::new(r)),
                Poll::Pending => Poll::Pending
            }
        }
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

        #[cfg(feature = "async")] awaiting_promise: None
    })
}
