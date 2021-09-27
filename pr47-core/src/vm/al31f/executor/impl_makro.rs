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
