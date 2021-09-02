use std::ptr::NonNull;

use unchecked_unwrap::UncheckedUnwrap;

use crate::data::Value;
use crate::data::exception::{CheckedException, Exception, UncheckedException};
use crate::ds::object::Object;
use crate::ffi::sync_fn::Function as FFIFunction;
use crate::util::mem::FatPointer;
use crate::util::serializer::Serializer;
use crate::vm::al31f::{AL31F, Combustor};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::insc::Insc;
use crate::vm::al31f::stack::{Stack, StackSlice, FrameInfo};

#[cfg(feature = "bench")] use crate::defer;
#[cfg(feature = "bench")] use crate::util::defer::Defer;
use crate::data::value_typed::{INT_TYPE_TAG, FLOAT_TYPE_TAG};
use crate::data::wrapper::DynBase;
use std::any::TypeId;

include!("impl_makro.rs");

pub struct VMThread<A: Alloc> {
    #[cfg(feature = "async")]
    vm: Serializer<AL31F<A>>,
    #[cfg(not(feature = "async"))]
    vm: AL31F<A>,

    program: NonNull<CompiledProgram<A>>,
    stack: Stack
}

#[must_use = "VM thread are effective iff a function gets run on it"]
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

#[inline(never)]
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

#[inline(never)]
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

    thread.vm.get_shared_data_mut().alloc.set_gc_allowed(true);

    let program: &CompiledProgram<A> = thread.program.as_ref();

    let compiled_function: &CompiledFunction = &thread.program.as_ref().functions[func_id];

    if compiled_function.arg_count != args.len() {
        let exception: UncheckedException = UncheckedException::ArgCountMismatch {
            func_id, expected: compiled_function.arg_count, got: args.len()
        };
        return Err(Exception::unchecked_exc(exception));
    }

    let mut slice: StackSlice =
        thread.stack.ext_func_call_grow_stack(func_id, compiled_function.stack_size, args);
    let mut insc_ptr: usize = compiled_function.start_addr;

    let mut ffi_args: Vec<Value> = Vec::with_capacity(8);
    let mut ffi_rets: Vec<*mut Value> = Vec::with_capacity(3);

    loop {
        #[cfg(not(debug_assertions))]
        let insc: &Insc = program.code.get_unchecked(insc_ptr);
        #[cfg(debug_assertions)]
        let insc: &Insc = &program.code[insc_ptr];

        insc_ptr += 1;
        match insc {
            Insc::AddInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, +],
            Insc::AddFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, +],
            Insc::AddAny(src1, src2, dst) => {
                let src1: Value = slice.get_value(*src1);
                let src2: Value = slice.get_value(*src2);
                if !src1.is_value() || !src2.is_value() {
                    if !src1.is_container() && !src2.is_container() {
                        let src1: *mut dyn DynBase = src1.get_as_dyn_base();
                        let src2: *mut dyn DynBase = src2.get_as_dyn_base();
                        if (*src1).dyn_type_id() == TypeId::of::<String>()
                            && (*src2).dyn_type_id() == TypeId::of::<String>() {
                            let src1: *const String = src1 as *mut String as *const _;
                            let src2: *const String = src2 as *mut String as *const _;
                            let result: String = format!("{}{}", *src1, *src2);
                            let result: Value = Value::new_owned(result);
                            thread.vm.get_shared_data_mut().alloc.add_managed(result.ptr_repr);
                            slice.set_value(*dst, result);
                            continue;
                        }
                    }

                    // TODO resolve overloaded call
                    let exception: UncheckedException = UncheckedException::InvalidBinaryOp {
                        bin_op: '+', lhs: src1, rhs: src2
                    };
                    return Err(unchecked_exception_unwind_stack(
                        exception, &mut thread.stack, insc_ptr
                    ));
                }

                if src1.vt_data.tag & INT_TYPE_TAG != 0 && src2.vt_data.tag & INT_TYPE_TAG != 0 {
                    slice.set_value(*dst, Value::new_int(
                        src1.vt_data.inner.int_value + src2.vt_data.inner.int_value
                    ))
                } else if src1.vt_data.tag & FLOAT_TYPE_TAG != 0
                    && src2.vt_data.tag & FLOAT_TYPE_TAG != 0
                {
                    slice.set_value(*dst, Value::new_float(
                        src1.vt_data.inner.float_value + src2.vt_data.inner.float_value
                    ))
                } else {
                    let exception: UncheckedException = UncheckedException::InvalidBinaryOp {
                        bin_op: '+', lhs: src1, rhs: src2
                    };
                    return Err(unchecked_exception_unwind_stack(
                        exception, &mut thread.stack, insc_ptr
                    ));
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
            Insc::SubInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, -],
            Insc::SubFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, -],
            Insc::SubAny(_, _, _) => {}
            Insc::MulInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, *],
            Insc::MulFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, *],
            Insc::MulAny(_, _, _) => {}
            Insc::DivInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, /],
            Insc::DivFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, /],
            Insc::DivAny(_, _, _) => {}
            Insc::ModInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, %],
            Insc::ModAny(_, _, _) => {}
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
            Insc::LtAny(_, _, _) => {}
            Insc::GtInt(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >, i64, int_value],
            Insc::GtFloat(src1, src2, dst) =>
                impl_rel_op![slice, src1, src2, dst, >, f64, float_value],
            Insc::GtAny(_, _, _) => {}
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
            Insc::BNotInt(_, _) => {}
            Insc::BNotAny(_, _) => {}
            Insc::NegInt(_, _) => {}
            Insc::NegFloat(_, _) => {}
            Insc::NegAny(_, _) => {}
            Insc::AndBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, &],
            Insc::AndAny(_, _, _) => {}
            Insc::OrBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, |],
            Insc::OrAny(_, _, _) => {}
            Insc::XorBool(src1, src2, dst) => impl_bool_binop![slice, src1, src2, dst, ^],
            Insc::XorAny(_, _, _) => {}
            Insc::NotBool(_, _) => {}
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
            }
            Insc::NullCheck(_) => {}
            Insc::TypeCheck(src, _tyck_info) => {
                let _src: Value = slice.get_value(*src);
                todo!();
            }
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
            }
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
            }
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
            }
            Insc::FFICallTyck(ffi_func_id, args, ret_value_locs) => {
                let ffi_function: &Box<dyn FFIFunction<Combustor<A>>>
                    = &program.ffi_funcs[*ffi_func_id];

                for arg /*: &usize*/ in args.iter() {
                    ffi_args.push(slice.get_value(*arg));
                }
                for ret_value_loc /*: &usize*/ in ret_value_locs.iter() {
                    ffi_rets.push(slice.get_value_mut_ref(*ret_value_loc));
                }
                let mut combustor: Combustor<A> =
                    Combustor::new(NonNull::from(thread.vm.get_shared_data_mut()));

                if let Some(_ /*: Exception*/) =
                    ffi_function.call_tyck(&mut combustor, &ffi_args, &mut ffi_rets)
                {
                    // is this checked exception or unchecked exception?
                    // or we handle them altogether?
                    todo!();
                    // let (new_slice, insc_ptr_next): (StackSlice, usize) =
                    //     checked_exception_unwind_stack(
                    //         thread.vm.get_shared_data_mut(),
                    //         &program,
                    //         exception,
                    //         &mut thread.stack,
                    //         insc_ptr
                    //     )?;
                    // slice = new_slice;
                    // insc_ptr = insc_ptr_next;
                    // continue;
                }

                ffi_args.clear();
                ffi_rets.clear();
            }
            #[cfg(feature = "optimized-rtlc")]
            Insc::FFICallRtlc(_, _, _) => {}
            Insc::FFICall(_, _, _) => {}
            Insc::FFICallAsyncTyck(_, _, _) => {}
            #[cfg(feature = "optimized-rtlc")]
            Insc::FFICallAsync(_, _, _) => {}
            #[cfg(feature = "no-rtlc")]
            Insc::FFICallAsyncUnchecked(_, _, _) => {}
            Insc::Await(_, _) => {}
            Insc::Raise(exception_ptr) => {
                let exception: Value = slice.get_value(*exception_ptr);
                let (new_slice, insc_ptr_next): (StackSlice, usize) =
                    checked_exception_unwind_stack(
                        thread.vm.get_shared_data_mut(),
                        &program,
                        exception,
                        &mut thread.stack,
                        insc_ptr
                    )?;
                slice = new_slice;
                insc_ptr = insc_ptr_next;
                continue;
            }
            Insc::JumpIfTrue(condition, dest) => {
                let condition: bool = slice.get_value(*condition).vt_data.inner.bool_value;
                if condition {
                    insc_ptr = *dest;
                }
            }
            Insc::JumpIfFalse(condition, dest) => {
                let condition: bool = slice.get_value(*condition).vt_data.inner.bool_value;
                if !condition {
                    insc_ptr = *dest;
                }
            }
            Insc::Jump(dest) => {
                insc_ptr = *dest;
            }
            Insc::CreateObject(dest) => {
                let object: Object = Object::new();
                let object: Value = Value::new_owned(object);
                thread.vm.get_shared_data_mut().alloc.add_managed(object.ptr_repr);
                slice.set_value(*dest, object);
            }
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
