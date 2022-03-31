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
    ($slice:ident, $src1:ident, $src2:ident, $dst:ident, $fn:ident) => {
        {
            let src1: i64 = $slice.get_value(*$src1).vt_data.inner.int_value;
            let src2: i64 = $slice.get_value(*$src2).vt_data.inner.int_value;
            $slice.set_value(*$dst, Value::new_int(i64::$fn(src1, src2)));
        }
    };
    ($slice:ident, $src1:ident, $src2:ident, $dst:ident, $op:tt) => {
        {
            let src1: i64 = $slice.get_value(*$src1).vt_data.inner.int_value;
            let src2: i64 = $slice.get_value(*$src2).vt_data.inner.int_value;
            $slice.set_value(*$dst, Value::new_int(src1 $op src2));
        }
    };
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

macro_rules! impl_checked_bin_op {
    (
        $slice:ident,
        $src1:ident,
        $src2:ident,
        $dst:ident,
        $checked_op:expr,
        $thread:expr,
        $insc_ptr:expr
    ) => {
        {
            let src1: Value = $slice.get_value(*$src1);
            let src2: Value = $slice.get_value(*$src2);
            let dst: &mut Value = &mut *$slice.get_value_mut_ref(*$dst);
            if let Err(e /*: UncheckedException*/) = $checked_op(src1, src2, dst) {
                return Poll::Ready(
                    Err(unchecked_exception_unwind_stack(e, &mut $thread.stack, $insc_ptr))
                );
            }
        }
    }
}

macro_rules! impl_checked_unary_op {
    ($slice:ident, $src:ident, $dst:ident, $checked_op:expr, $thread:expr, $insc_ptr:expr) => {
        {
            let src: Value = $slice.get_value(*$src);
            let dst: &mut Value = &mut *$slice.get_value_mut_ref(*$dst);
            if let Err(e /*: UncheckedException*/) = $checked_op(src, dst) {
                return Poll::Ready(
                    Err(unchecked_exception_unwind_stack(e, &mut $thread.stack, $insc_ptr))
                );
            }
        }
    }
}

macro_rules! impl_checked_cast_op {
    ($slice:ident, $src:ident, $dst:ident, $checked_op:expr, $thread:expr, $insc_ptr:expr) => {
        impl_checked_unary_op![$slice, $src, $dst, $checked_op, $thread, $insc_ptr]
    }
}
