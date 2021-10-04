use std::any::TypeId;
use std::ptr::NonNull;

use unchecked_unwrap::UncheckedUnwrap;

use crate::collections::object::Object;
use crate::data::Value;
use crate::data::exception::{CheckedException, Exception, UncheckedException};
use crate::data::value_typed::INT_TYPE_TAG;
use crate::ffi::sync_fn::Function as FFIFunction;
use crate::util::either::Either;
use crate::util::mem::FatPointer;
use crate::vm::al31f::{AL31F, Combustor};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::executor::checked_ops::{
    checked_add,
    checked_div,
    checked_gt,
    checked_lt,
    checked_mod,
    checked_mul,
    checked_sub
};
use crate::vm::al31f::insc::Insc;
use crate::vm::al31f::stack::{Stack, StackSlice, FrameInfo};

#[cfg(feature = "async")] use crate::data::wrapper::{Wrapper, OwnershipInfo};
#[cfg(feature = "async")] use crate::ffi::async_fn::{AsyncReturnType, Promise};
#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncFunction as FFIAsyncFunction;
#[cfg(feature = "async")] use crate::util::serializer::Serializer;
#[cfg(feature = "async")] use crate::vm::al31f::AsyncCombustor;
#[cfg(feature = "bench")] use crate::defer;
#[cfg(feature = "bench")] use crate::util::defer::Defer;

include!("get_vm_makro.rs");
include!("impl_makro.rs");

pub struct VMThread<A: Alloc> {
    #[cfg(feature = "async")]
    pub vm: Serializer<AL31F<A>>,
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
    let ret = Box::new(VMThread {
        vm: Serializer::new(AL31F::new(alloc)).await,
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

pub async unsafe fn vm_thread_run_function<A: Alloc>(
    thread: &mut VMThread<A>,
    func_id: usize,
    args: &[Value]
) -> Result<Vec<Value>, Exception> {
    #[cfg(feature = "bench")] let start_time: std::time::Instant = std::time::Instant::now();
    #[cfg(feature = "bench")] defer!(move || {
        let end_time: std::time::Instant = std::time::Instant::now();
        eprintln!("Time consumed: {}ms", (end_time - start_time).as_millis());
    });

    get_vm!(thread).alloc.set_gc_allowed(true);

    let program: &CompiledProgram<A> = thread.program.as_ref();
    let compiled_function: &CompiledFunction = &program.functions[func_id];
    if compiled_function.arg_count != args.len() {
        let exception: UncheckedException = UncheckedException::ArgCountMismatch {
            func_id, expected: compiled_function.arg_count, got: args.len()
        };
        return Err(Exception::unchecked_exc(exception));
    }

    let mut slice: StackSlice =
        thread.stack.ext_func_call_grow_stack(func_id, compiled_function.stack_size, args);
    let mut insc_ptr: usize = compiled_function.start_addr;

    let mut ffi_args: Vec<Value> = vec![Value::new_null(); 32];
    let mut ffi_rets: Vec<*mut Value> = vec![std::ptr::null_mut(); 8];

    loop {
        #[cfg(not(debug_assertions))]
        let insc: &Insc = program.code.get_unchecked(insc_ptr);
        #[cfg(debug_assertions)]
        let insc: &Insc = &program.code[insc_ptr];

        insc_ptr += 1;
        match insc {
            Insc::AddInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, wrapping_add],
            Insc::AddFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, +],
            Insc::AddAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_add(thread, src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
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
            Insc::SubAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_sub(src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
            Insc::MulInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, wrapping_mul],
            Insc::MulFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, *],
            Insc::MulAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_mul(src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
            Insc::DivInt(src1, src2, dst) => {
                let src1: i64 = slice.get_value(*src1).vt_data.inner.int_value;
                let src2: i64 = slice.get_value(*src2).vt_data.inner.int_value;
                if let Some(result) = i64::checked_div(src1, src2) {
                    slice.set_value(*dst, Value::new_int(result))
                } else {
                    return Err(unchecked_exception_unwind_stack(
                        UncheckedException::DivideByZero, &mut thread.stack, insc_ptr
                    ))
                }
            },
            Insc::DivFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, /],
            Insc::DivAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_div(src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
            Insc::ModInt(src1, src2, dst) => {
                let src1: i64 = slice.get_value(*src1).vt_data.inner.int_value;
                let src2: i64 = slice.get_value(*src2).vt_data.inner.int_value;
                if let Some(result) = i64::checked_rem(src1, src2) {
                    slice.set_value(*dst, Value::new_int(result))
                } else {
                    return Err(unchecked_exception_unwind_stack(
                        UncheckedException::DivideByZero, &mut thread.stack, insc_ptr
                    ))
                }
            },
            Insc::ModAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_mod(src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
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
                let src1: FatPointer = slice.get_value(*src1).ptr_repr;
                let src2: FatPointer = slice.get_value(*src2).ptr_repr;
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
                let src1: FatPointer = slice.get_value(*src1).ptr_repr;
                let src2: FatPointer = slice.get_value(*src2).ptr_repr;
                slice.set_value(*dst, Value::new_bool(src1 != src2));
            },
            Insc::LtInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <, i64, int_value],
            Insc::LtFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <, f64, float_value],
            Insc::LtAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_lt(src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
            Insc::GtInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >, i64, int_value],
            Insc::GtFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >, f64, float_value],
            Insc::GtAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                let dst: &mut Value = &mut *slice.get_value_mut_ref(*dst);
                if let Err(e /*: UncheckedException*/) = checked_gt(src1, src2, dst) {
                    return Err(unchecked_exception_unwind_stack(e, &mut thread.stack, insc_ptr));
                }
            },
            Insc::LeInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <=, i64, int_value],
            Insc::LeFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, <=, f64, float_value],
            Insc::LeAny(_, _, _) => {}
            Insc::GeInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >=, i64, int_value],
            Insc::GeFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >=, f64, float_value],
            Insc::GeAny(_, _, _) => {}
            Insc::BAndInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, &],
            Insc::BAndAny(_, _, _) => {}
            Insc::BOrInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, |],
            Insc::BOrAny(_, _, _) => {}
            Insc::BXorInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, ^],
            Insc::BXorAny(_, _, _) => {}
            Insc::BNotInt(src, dst) => {
                let src: u64 = slice.get_value(*src).vt_data.inner.repr;
                slice.set_value(*dst, Value::new_raw_value(INT_TYPE_TAG, u64::reverse_bits(src)));
            },
            Insc::BNotAny(_, _) => {}
            Insc::NegInt(src, dst) => {
                let src: i64 = slice.get_value(*src).vt_data.inner.int_value;
                slice.set_value(*dst, Value::new_int(i64::wrapping_neg(src)));
            },
            Insc::NegFloat(src, dst) => {
                let src: f64 = slice.get_value(*src).vt_data.inner.float_value;
                slice.set_value(*dst, Value::new_float(-src));
            },
            Insc::NegAny(_, _) => {}
            Insc::AndBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, &],
            Insc::AndAny(_, _, _) => {}
            Insc::OrBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, |],
            Insc::OrAny(_, _, _) => {}
            Insc::XorBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, ^],
            Insc::XorAny(_, _, _) => {}
            Insc::NotBool(src, dst) => {
                let src: bool = slice.get_value(*src).vt_data.inner.bool_value;
                slice.set_value(*dst, Value::new_bool(!src));
            },
            Insc::NotAny(_, _) => {}
            Insc::ShlInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, <<],
            Insc::ShlAny(_, _, _) => {}
            Insc::ShrInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, >>],
            Insc::ShrAny(_, _, _) => {}
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
            Insc::CastAnyInt(_, _) => {}
            Insc::CastIntFloat(src, dst) =>
                impl_cast_op![slice, src, dst, i64, f64, int_value, new_float],
            Insc::CastAnyFloat(_, _) => {}
            Insc::CastAnyChar(_, _) => {}
            Insc::IsNull(src, dst) => {
                let src: Value = slice.get_value(*src);
                slice.set_value(*dst, Value::new_bool(src.is_null()));
            },
            Insc::NullCheck(src) => {
                let src: Value = slice.get_value(*src);
                if src.is_null() {
                    return Err(unchecked_exception_unwind_stack(
                        UncheckedException::UnexpectedNull { value: src },
                        &mut thread.stack,
                        insc_ptr
                    ))
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
                slice = thread.stack.func_call_grow_stack(
                    *func_id,
                    compiled.stack_size,
                    args,
                    NonNull::from(&rets[..]),
                    insc_ptr
                );
                insc_ptr = compiled.start_addr;
            },
            Insc::CallTyck(_, _, _) => {}
            Insc::CallPtr(_, _, _) => {}
            Insc::CallPtrTyck(_, _, _) => {}
            Insc::CallOverload(_, _, _) => {}
            Insc::ReturnNothing => {
                if let Some((prev_stack_slice, ret_addr)) =
                    thread.stack.done_func_call_shrink_stack0()
                {
                    insc_ptr = ret_addr;
                    slice = prev_stack_slice;
                } else {
                    return Ok(vec![]);
                }
            },
            Insc::ReturnOne(ret_value) => {
                if let Some((prev_stack_slice, ret_addr)) =
                    thread.stack.done_func_call_shrink_stack1(*ret_value)
                {
                    insc_ptr = ret_addr;
                    slice = prev_stack_slice;
                } else {
                    return Ok(vec![slice.get_value(*ret_value)]);
                }
            },
            Insc::Return(ret_values) => {
                if let Some((prev_stack_slice, ret_addr)) =
                    thread.stack.done_func_call_shrink_stack(&ret_values)
                {
                    insc_ptr = ret_addr;
                    slice = prev_stack_slice;
                } else {
                    let mut ret_vec: Vec<Value> = Vec::with_capacity(ret_values.len());
                    for ret_value_loc in ret_values.iter() {
                        ret_vec.push(slice.get_value(*ret_value_loc));
                    }
                    return Ok(ret_vec);
                }
            },
            Insc::FFICallTyck(ffi_func_id, args, ret_value_locs) => {
                let ffi_function: &Box<dyn FFIFunction<Combustor<A>>>
                    = &program.ffi_funcs[*ffi_func_id];

                for i /*: usize*/ in 0..args.len() {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }
                ffi_args.set_len(args.len());

                for i /*: usize*/ in 0..ret_value_locs.len() {
                    let ret_value_loc_idx: usize = *ret_value_locs.get_unchecked(i);
                    *ffi_rets.get_unchecked_mut(i) = slice.get_value_mut_ref(ret_value_loc_idx);
                }
                ffi_rets.set_len(ret_value_locs.len());

                let mut combustor: Combustor<A> = Combustor::new(NonNull::from(get_vm!(thread)));

                if let Err(e /*: FFIException*/) =
                    ffi_function.call_tyck(&mut combustor, &ffi_args, &mut ffi_rets)
                {
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
                            slice = new_slice;
                            insc_ptr = insc_ptr_next;
                        },
                        Either::Right(unchecked) => {
                            return Err(unchecked_exception_unwind_stack(
                                unchecked, &mut thread.stack, insc_ptr
                            ));
                        }
                    }
                }
            },
            #[cfg(feature = "optimized-rtlc")]
            Insc::FFICallRtlc(ffi_func_id, args, ret_value_locs) => {
                let ffi_function: &Box<dyn FFIFunction<Combustor<A>>>
                    = &program.ffi_funcs[*ffi_func_id];

                for i /*: usize*/ in 0..args.len() {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }
                ffi_args.set_len(args.len());

                for i /*: usize*/ in 0..ret_value_locs.len() {
                    let ret_value_loc_idx: usize = *ret_value_locs.get_unchecked(i);
                    *ffi_rets.get_unchecked_mut(i) = slice.get_value_mut_ref(ret_value_loc_idx);
                }
                ffi_rets.set_len(ret_value_locs.len());

                let mut combustor: Combustor<A> = Combustor::new(NonNull::from(get_vm!(thread)));

                if let Err(e /*: FFIException*/) =
                    ffi_function.call_rtlc(&mut combustor, &ffi_args, &mut ffi_rets)
                {
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
                            slice = new_slice;
                            insc_ptr = insc_ptr_next;
                        },
                        Either::Right(unchecked) => {
                            return Err(unchecked_exception_unwind_stack(
                                unchecked, &mut thread.stack, insc_ptr
                            ));
                        }
                    }
                }
            },
            Insc::FFICall(_, _, _) => {}
            Insc::FFICallAsyncTyck(_, _, _) => {}
            #[cfg(all(feature = "optimized-rtlc", feature = "async"))]
            Insc::FFICallAsync(async_ffi_func_id, args, ret) => {
                let async_ffi_function: &Box<dyn FFIAsyncFunction<AsyncCombustor<A>>>
                    = &program.async_ffi_funcs[*async_ffi_func_id];

                for i /*: usize*/ in 0..args.len() {
                    let arg_idx: usize = *args.get_unchecked(i);
                    *ffi_args.get_unchecked_mut(i) = slice.get_value(arg_idx);
                }
                ffi_args.set_len(args.len());

                let mut combustor: AsyncCombustor<A> = AsyncCombustor::new(
                    NonNull::from(&thread.vm)
                );

                match async_ffi_function.call_rtlc(&mut combustor, &ffi_args) {
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
                                slice = new_slice;
                                insc_ptr = insc_ptr_next;
                            },
                            Either::Right(unchecked) => {
                                return Err(unchecked_exception_unwind_stack(
                                    unchecked, &mut thread.stack, insc_ptr
                                ));
                            }
                        }
                    }
                }
            },
            #[cfg(feature = "no-rtlc")]
            Insc::FFICallAsyncUnchecked(_, _, _) => {}
            #[cfg(feature = "async")]
            Insc::Await(promise, dests) => {
                let promise: Value = slice.get_value(*promise);
                let wrapper: *mut Wrapper<()> = promise.ptr_repr.ptr as *mut Wrapper<()>;
                if (*wrapper).ownership_info == OwnershipInfo::MovedToRust as u8 {
                    return Err(unchecked_exception_unwind_stack(
                        UncheckedException::AlreadyAwaited { promise },
                        &mut thread.stack,
                        insc_ptr
                    ));
                }

                let promise: Promise = promise.move_out::<Promise>();
                (*wrapper).ownership_info = OwnershipInfo::MovedToRust as u8;

                let AsyncReturnType(result) = thread.vm.co_await(promise.await_promise()).await;
                match result {
                    Ok(values) => {
                        debug_assert_eq!(values.len(), dests.len());
                        for i in 0..dests.len() {
                            let dest: usize = *dests.get_unchecked(i);
                            slice.set_value(dest, *values.get_unchecked(i));
                        }
                    },
                    Err(e) => {
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
                                slice = new_slice;
                                insc_ptr = insc_ptr_next;
                            },
                            Either::Right(unchecked) => {
                                return Err(unchecked_exception_unwind_stack(
                                    unchecked, &mut thread.stack, insc_ptr
                                ));
                            }
                        }
                    }
                }
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
                slice = new_slice;
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
            Insc::CreateObject(dest) => {
                let object: Object = Object::new();
                let object: Value = Value::new_owned(object);
                get_vm!(thread).alloc.add_managed(object.ptr_repr);
                slice.set_value(*dest, object);
            },
            Insc::CreateContainer(_, _, _) => {}
            Insc::VecIndex(_, _, _) => {}
            Insc::VecIndexPut(_, _, _) => {}
            Insc::VecPush(_, _) => {}
            Insc::VecPop(_, _) => {}
            Insc::VecFirst(_, _) => {}
            Insc::VecLast(_, _) => {}
            Insc::VecLen(_, _) => {}
            Insc::StrConcat(_, _, _) => {}
            Insc::StrAppend(_, _) => {}
            Insc::StrIndex(_, _, _) => {}
            Insc::StrLen(_, _) => {}
            Insc::ObjectGet(_, _, _) => {}
            Insc::ObjectGetDyn(_, _, _) => {}
            Insc::ObjectPut(_, _, _) => {}
            Insc::ObjectPutDyn(_, _, _) => {}
        }
    }
}
