//! # `executor.rs`: core executor of AL31F
//!
//! ## ⚠️⚠️⚠️ Develop stage note ⚠️⚠️⚠
//! By this time the developers don't know what's the correct abstraction. This `executor` module
//! is temporary, maybe just here for testing. Project structure may change a lot in further days.

use std::ptr::NonNull;

use crate::data::Value;
use crate::data::exception::{Exception, UncheckedException};
use crate::util::serializer::Serializer;
use crate::vm::al31f::{AL31F, VMThread};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::insc::Insc;
use crate::vm::al31f::stack::{Stack, StackSlice};
use crate::util::mem::FatPointer;

#[must_use = "VM thread are effective iff a function gets run on it"]
#[cfg(feature = "async")]
pub async fn create_vm_main_thread<A: Alloc>(
    alloc: A,
    program: &CompiledProgram<A>
) -> VMThread<A> {
    let ret = VMThread {
        vm: Serializer::new(AL31F { alloc }).await,
        program: NonNull::from(program),
        stack: Stack::new()
    };
    unsafe { ret.vm.get_shared_data_mut().alloc.add_stack(&ret.stack) };
    ret
}

#[cfg(feature = "async")]
pub async unsafe fn vm_thread_run_function<A: Alloc>(
    thread: &mut VMThread<A>,
    func_ptr: usize,
    args: &[Value]
) -> Result<Vec<Value>, Exception> {
    let program: &CompiledProgram<A> = thread.program.as_ref();
    let stack: &mut Stack = &mut thread.stack;

    let compiled_function: &CompiledFunction = &thread.program.as_ref().functions[func_ptr];

    if compiled_function.arg_count != args.len() {
        let exception: UncheckedException = UncheckedException::ArgCountMismatch {
            func_ptr, expected: compiled_function.arg_count, got: args.len()
        };
        return Err(Exception::UncheckedException(exception))
    }

    let mut slice: StackSlice =
        stack.ext_func_call_grow_stack(compiled_function.stack_size, args);
    let mut insc_ptr: usize = compiled_function.start_addr;

    macro_rules! impl_value_typed_binop {
        (
            $slice:ident,
            $src1:ident,
            $src2:ident,
            $dst:ident,
            $type:ty,
            $op:tt,
            $value:ident,
            $value_ctor:ident
        ) => {
            let src1: $type = $slice.get_value(*$src1).vt_data.inner.$value;
            let src2: $type = $slice.get_value(*$src2).vt_data.inner.$value;
            $slice.set_value(*$dst, Value::$value_ctor(src1 $op src2));
        }
    }

    macro_rules! impl_cast_op {
        (
            $slice:ident,
            $src:ident,
            $dst:ident,
            $src_type:ty,
            $dst_type:ty,
            $from_value:ident,
            $value_ctor:ident
        ) => {
            {
                let src: $src_type = $slice.get_value(*$src).vt_data.inner.$from_value;
                let casted: $dst_type = src as _;
                $slice.set_value(*$dst, Value::$value_ctor(casted));
            }
        }
    }

    macro_rules! impl_int_binop {
        ($slice:ident, $src1:ident, $src2:ident, $dst:ident, $op:tt) => {
            {
                impl_value_typed_binop![$slice, $src1, $src2, $dst, i64, $op, int_value, new_int];
            }
        }
    }

    macro_rules! impl_float_binop {
        ($slice:ident, $src1:ident, $src2:ident, $dst:ident, $op:tt) => {
            {
                impl_value_typed_binop![
                    $slice, $src1, $src2, $dst, f64, $op, float_value, new_float
                ];
            }
        }
    }

    macro_rules! impl_bool_binop {
        ($slice:ident, $src1:ident, $src2:ident, $dst:ident, $op:tt) => {
            {
                impl_value_typed_binop![
                    $slice, $src1, $src2, $dst, bool, $op, bool_value, new_bool
                ];
            }
        }
    }

    macro_rules! impl_rel_op {
        ($slice:ident, $src1:ident, $src2:ident, $dst:ident, $rel:tt, $type:ty, $value:ident) => {
            {
                impl_value_typed_binop![$slice, $src1, $src2, $dst, $type, $rel, $value, new_bool];
            }
        }
    }

    loop {
        #[cfg(debug_assertions)]
        let insc: &Insc = &program.code[insc_ptr];
        #[cfg(not(debug_assertions))]
        let insc: &Insc = unsafe { program.code.get_unchecked(insc_ptr) };

        insc_ptr += 1;
        match insc {
            Insc::AddInt(src1, src2, dst) => impl_int_binop![slice, src1, src2, dst, +],
            Insc::AddFloat(src1, src2, dst) => impl_float_binop![slice, src1, src2, dst, +],
            Insc::AddAny(_, _, _) => { todo!() },
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
                let src1: usize = slice.get_value(*src1).ptr_repr.trivia;
                let src2: usize = slice.get_value(*src2).ptr_repr.trivia;
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
                let src1: usize = slice.get_value(*src1).ptr_repr.trivia;
                let src2: usize = slice.get_value(*src2).ptr_repr.trivia;
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
            Insc::CastCharInt(src, dst) =>
                impl_cast_op![slice, src, dst, char, i64, char_value, new_int],
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
            Insc::Call(_, _, _) => {}
            Insc::CallTyck(_, _, _) => {}
            Insc::CallPtr(_, _, _) => {}
            Insc::CallPtrTyck(_, _, _) => {}
            Insc::CallOverload(_, _, _) => {}
            Insc::Return(ret_values) => {
                if let Some((prev_stack_slice, ret_addr)) =
                    stack.done_func_call_shrink_stack(&ret_values)
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
            Insc::FFICallTyck(_, _, _) => {}
            Insc::FFICallRtlc(_, _, _) => {}
            Insc::FFICall(_, _, _) => {}
            Insc::FFICallAsyncTyck(_, _, _) => {}
            #[cfg(feature = "optimized-rtlc")]
            Insc::FFICallAsync(_, _, _) => {}
            #[cfg(feature = "no-rtlc")]
            Insc::FFICallAsyncUnchecked(_, _, _) => {}
            Insc::Await(_, _) => {}
            Insc::JumpIfTrue(_, _) => {}
            Insc::JumpIfFalse(_, _) => {}
            Insc::Jump(_) => {}
            Insc::CreateObject => {}
            Insc::CreateContainer(_, _) => {}
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

#[cfg(not(feature = "async"))]
pub fn vm_thread_run_function<A: Alloc>(
    thread: &mut VMThread<A>,
    function_ptr: usize,
    args: &[Value]
) -> Result<Vec<Value>, Exception> {
    todo!()
}