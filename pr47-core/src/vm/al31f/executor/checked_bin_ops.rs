use std::any::TypeId;

use crate::data::Value;
use crate::data::exception::UncheckedException;
use crate::data::value_typed::{BOOL_TYPE_TAG, FLOAT_TYPE_TAG, INT_TYPE_TAG, VALUE_TYPE_TAG_MASK};
use crate::data::wrapper::{DynBase, Wrapper, OWN_INFO_READ_MASK};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::executor::VMThread;

include!("get_vm_makro.rs");

#[inline(never)] pub unsafe fn checked_add<A: Alloc>(
    thread: &mut VMThread<A>,
    src1: Value,
    src2: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    debug_assert!(!src1.is_null());
    debug_assert!(!src2.is_null());

    if !src1.is_value() || !src2.is_value() {
        if !src1.is_container() && !src2.is_container() {
            let dyn1: *mut dyn DynBase = src1.get_as_dyn_base();
            let wrapper1: *mut Wrapper<()> = dyn1 as *mut Wrapper<()>;
            if (*wrapper1).ownership_info & OWN_INFO_READ_MASK != 0 {
                return Err(UncheckedException::OwnershipCheckFailure {
                    object: src1, expected_mask: OWN_INFO_READ_MASK
                })
            }

            let dyn2: *mut dyn DynBase = src2.get_as_dyn_base();
            let wrapper2: *mut Wrapper<()> = dyn2 as *mut Wrapper<()>;
            if (*wrapper2).ownership_info & OWN_INFO_READ_MASK != 0 {
                return Err(UncheckedException::OwnershipCheckFailure {
                    object: src2, expected_mask: OWN_INFO_READ_MASK
                })
            }

            if (*dyn1).dyn_type_id() == TypeId::of::<String>()
                && (*dyn2).dyn_type_id() == TypeId::of::<String>() {
                let src1: *const String = src1.get_as_mut_ptr_norm::<String>() as *const _;
                let src2: *const String = src2.get_as_mut_ptr_norm::<String>() as *const _;
                let result: String = format!("{}{}", *src1, *src2);
                let result: Value = Value::new_owned(result);
                get_vm!(thread).alloc.add_managed(result.ptr_repr);
                *dest = result;
                return Ok(())
            }
        }

        return Err(UncheckedException::InvalidBinaryOp { bin_op: '+', lhs: src1, rhs: src2 });
    }

    let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

    if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
        *dest = Value::new_int(src1.vt_data.inner.int_value + src2.vt_data.inner.int_value);
        Ok(())
    } else if src1_tag == FLOAT_TYPE_TAG && src2_tag == FLOAT_TYPE_TAG {
        *dest = Value::new_float(src1.vt_data.inner.float_value + src2.vt_data.inner.float_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidBinaryOp { bin_op: '+', lhs: src1, rhs: src2 })
    }
}

#[inline(never)] pub unsafe fn checked_sub(
    src1: Value,
    src2: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src1.is_value() || !src2.is_value() {
        return Err(UncheckedException::InvalidBinaryOp { bin_op: '-', lhs: src1, rhs: src2 });
    }

    let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

    if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
        *dest = Value::new_int(src1.vt_data.inner.int_value - src2.vt_data.inner.int_value);
        Ok(())
    } else if src1_tag == FLOAT_TYPE_TAG && src2_tag == FLOAT_TYPE_TAG {
        *dest = Value::new_float(src1.vt_data.inner.float_value - src2.vt_data.inner.float_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidBinaryOp { bin_op: '-', lhs: src1, rhs: src2 })
    }
}

#[inline(never)] pub unsafe fn checked_mul(
    src1: Value,
    src2: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src1.is_value() || !src2.is_value() {
        return Err(UncheckedException::InvalidBinaryOp { bin_op: '*', lhs: src1, rhs: src2 });
    }

    let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

    if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
        *dest = Value::new_int(src1.vt_data.inner.int_value * src2.vt_data.inner.int_value);
        Ok(())
    } else if src1_tag == FLOAT_TYPE_TAG && src2_tag == FLOAT_TYPE_TAG {
        *dest = Value::new_float(src1.vt_data.inner.float_value * src2.vt_data.inner.float_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidBinaryOp { bin_op: '*', lhs: src1, rhs: src2 })
    }
}

#[inline(never)] pub unsafe fn checked_div(
    src1: Value,
    src2: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src1.is_value() || !src2.is_value() {
        return Err(UncheckedException::InvalidBinaryOp { bin_op: '/', lhs: src1, rhs: src2 });
    }

    let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

    if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
        if let Some(result /*: UncheckedException*/) = i64::checked_div(
            src1.vt_data.inner.int_value, src2.vt_data.inner.int_value
        ) {
            *dest = Value::new_int(result);
        } else {
            return Err(UncheckedException::DivideByZero)
        }
        Ok(())
    } else if src1_tag == FLOAT_TYPE_TAG && src2_tag == FLOAT_TYPE_TAG {
        *dest = Value::new_float(src1.vt_data.inner.float_value / src2.vt_data.inner.float_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidBinaryOp { bin_op: '/', lhs: src1, rhs: src2 })
    }
}

#[inline(never)] pub unsafe fn checked_mod(
    src1: Value,
    src2: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src1.is_value() || !src2.is_value() {
        return Err(UncheckedException::InvalidBinaryOp { bin_op: '%', lhs: src1, rhs: src2 });
    }

    let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

    if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
        if let Some(result /*: UncheckedException*/) = i64::checked_rem(
            src1.vt_data.inner.int_value, src2.vt_data.inner.int_value
        ) {
            *dest = Value::new_int(result);
        } else {
            return Err(UncheckedException::DivideByZero)
        }
        Ok(())
    } else if src1_tag == FLOAT_TYPE_TAG && src2_tag == FLOAT_TYPE_TAG {
        *dest = Value::new_float(src1.vt_data.inner.float_value / src2.vt_data.inner.float_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidBinaryOp { bin_op: '%', lhs: src1, rhs: src2 })
    }
}

macro_rules! impl_checked_rel_op {
    ($fn_name:ident, $op_char:expr, $op:tt) => {
        #[inline(never)] pub unsafe fn $fn_name(
            src1: Value,
            src2: Value,
            dest: &mut Value
        ) -> Result<(), UncheckedException> {
            if !src1.is_value() || !src2.is_value() {
                return Err(UncheckedException::InvalidBinaryOp {
                    bin_op: $op_char, lhs: src1, rhs: src2
                });
            }

            let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
            let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

            if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
                *dest = Value::new_bool(
                    src1.vt_data.inner.int_value $op src2.vt_data.inner.int_value
                );
                Ok(())
            } else if src1_tag == FLOAT_TYPE_TAG && src2_tag == FLOAT_TYPE_TAG {
                *dest = Value::new_bool(
                    src1.vt_data.inner.float_value $op src2.vt_data.inner.float_value
                );
                Ok(())
            } else {
                Err(UncheckedException::InvalidBinaryOp { bin_op: $op_char, lhs: src1, rhs: src2 })
            }
        }
    }
}

impl_checked_rel_op![checked_lt, '<', <];
impl_checked_rel_op![checked_gt, '>', >];
impl_checked_rel_op![checked_ge, '≥', >=];
impl_checked_rel_op![checked_le, '≤', <=];

macro_rules! impl_checked_bitwise_op {
    ($fn_name:ident, $op_char:expr, $op:tt) => {
        #[inline(never)] pub unsafe fn $fn_name(
            src1: Value,
            src2: Value,
            dest: &mut Value
        ) -> Result<(), UncheckedException> {
            if !src1.is_value() || !src2.is_value() {
                return Err(UncheckedException::InvalidBinaryOp {
                    bin_op: $op_char, lhs: src1, rhs: src2
                });
            }

            let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
            let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

            if src1_tag == INT_TYPE_TAG && src2_tag == INT_TYPE_TAG {
                *dest = Value::new_raw_value(
                    INT_TYPE_TAG,
                    src1.vt_data.inner.repr $op src2.vt_data.inner.repr
                );
                Ok(())
            } else {
                Err(UncheckedException::InvalidBinaryOp { bin_op: $op_char, lhs: src1, rhs: src2 })
            }
        }
    }
}

impl_checked_bitwise_op![checked_bit_and, '&', &];
impl_checked_bitwise_op![checked_bit_or, '|', |];
impl_checked_bitwise_op![checked_bit_xor, '^', ^];
impl_checked_bitwise_op![checked_bit_shl, '⇇', <<];
impl_checked_bitwise_op![checked_bit_shr, '⇉', >>];

macro_rules! impl_checked_logic_op {
    ($fn_name:ident, $op_char:expr, $op:tt) => {
        #[inline(never)] pub unsafe fn $fn_name(
            src1: Value,
            src2: Value,
            dest: &mut Value
        ) -> Result<(), UncheckedException> {
            if !src1.is_value() || !src2.is_value() {
                return Err(UncheckedException::InvalidBinaryOp {
                    bin_op: $op_char, lhs: src1, rhs: src2
                });
            }

            let src1_tag: usize = src1.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
            let src2_tag: usize = src2.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);

            if src1_tag == BOOL_TYPE_TAG && src2_tag == BOOL_TYPE_TAG {
                *dest = Value::new_bool(
                    src1.vt_data.inner.bool_value $op src2.vt_data.inner.bool_value
                );
                Ok(())
            } else {
                Err(UncheckedException::InvalidBinaryOp { bin_op: $op_char, lhs: src1, rhs: src2 })
            }
        }
    }
}

impl_checked_logic_op![checked_logic_and, '∧', &];
impl_checked_logic_op![checked_logic_or, '∨', |];
impl_checked_logic_op![checked_logic_xor, '⊻', ^];
