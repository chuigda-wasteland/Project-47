use crate::data::Value;
use crate::data::exception::UncheckedException;
use crate::data::value_typed::{
    BOOL_TYPE_TAG,
    CHAR_TYPE_TAG,
    INT_TYPE_TAG,
    VALUE_TYPE_TAG_MASK,
    FLOAT_TYPE_TAG
};

#[inline(never)] pub unsafe fn cast_any_int(
    src: Value, dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src.is_value() {
        return Err(UncheckedException::InvalidCastOp { dest_type: "int", src });
    }

    match src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize) {
        INT_TYPE_TAG => *dest = src,
        CHAR_TYPE_TAG => *dest = Value::new_int(src.vt_data.inner.char_value as u32 as i64),
        FLOAT_TYPE_TAG => *dest = Value::new_int(src.vt_data.inner.float_value as i64),
        BOOL_TYPE_TAG => *dest = Value::new_int(if src.vt_data.inner.bool_value { 1 } else { 0 }),
        _ => unreachable!()
    }

    Ok(())
}

#[inline(never)] pub unsafe fn cast_any_float(
    src: Value, dest: &mut Value
) -> Result<(), UncheckedException> {
    if !src.is_value() {
        return Err(UncheckedException::InvalidCastOp { dest_type: "float", src });
    }

    match src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize) {
        FLOAT_TYPE_TAG => *dest = src,
        INT_TYPE_TAG => *dest = Value::new_float(src.vt_data.inner.int_value as f64),
        _ => return Err(UncheckedException::InvalidCastOp { dest_type: "float", src })
    }

    Ok(())
}

#[inline(never)] pub unsafe fn cast_any_char(
    src: Value, dest: &mut Value
) -> Result<(), UncheckedException> {
    match src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize) {
        CHAR_TYPE_TAG => *dest = src,
        // should we even implement this?
        // INT_TYPE_TAG => *dest = Value::new_char(src.vt_data.inner.int_value as u32 as char),
        _ => return Err(UncheckedException::InvalidCastOp { dest_type: "char", src })
    }

    Ok(())
}

#[inline(never)] pub unsafe fn cast_any_bool(
    src: Value, dest: &mut Value
) -> Result<(), UncheckedException> {
    match src.vt_data.tag & (VALUE_TYPE_TAG_MASK as usize) {
        BOOL_TYPE_TAG => *dest = src,
        INT_TYPE_TAG => *dest = Value::new_bool(src.vt_data.inner.int_value != 0),
        _ => return Err(UncheckedException::InvalidCastOp { dest_type: "bool", src })
    }

    Ok(())
}
