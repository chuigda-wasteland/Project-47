use crate::data::Value;
use crate::data::exception::UncheckedException;
use crate::data::value_typed::{BOOL_TYPE_TAG, INT_TYPE_TAG, VALUE_TYPE_TAG_MASK, FLOAT_TYPE_TAG};

#[inline(never)] pub unsafe fn checked_neg(
    src: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src.is_value() {
        return Err(UncheckedException::InvalidUnaryOp { unary_op: '-', src });
    }

    let src_tag: usize = src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    if src_tag == INT_TYPE_TAG {
        *dest = Value::new_int(-src.vt_data.inner.int_value);
        Ok(())
    } else if src_tag == FLOAT_TYPE_TAG {
        *dest = Value::new_float(-src.vt_data.inner.float_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidUnaryOp { unary_op: '-', src })
    }
}

#[inline(never)] pub unsafe fn checked_not(
    src: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src.is_value() {
        return Err(UncheckedException::InvalidUnaryOp { unary_op: '!', src });
    }

    let src_tag: usize = src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    if src_tag == BOOL_TYPE_TAG {
        *dest = Value::new_bool(!src.vt_data.inner.bool_value);
        Ok(())
    } else {
        Err(UncheckedException::InvalidUnaryOp { unary_op: '!', src })
    }
}


#[inline(never)] pub unsafe fn checked_bit_not(
    src: Value,
    dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src.is_value() {
        return Err(UncheckedException::InvalidUnaryOp { unary_op: '~', src });
    }

    let src_tag: usize = src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize);
    if src_tag == INT_TYPE_TAG {
        *dest = Value::new_raw_value(INT_TYPE_TAG, !src.vt_data.inner.repr);
        Ok(())
    } else {
        Err(UncheckedException::InvalidUnaryOp { unary_op: '~', src })
    }
}
