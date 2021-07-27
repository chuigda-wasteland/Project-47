use std::error::Error;

use crate::data::Value;

pub enum UncheckedException {
    ArgCountMismatch { func_ptr: usize, expected: usize, got: usize },
    InvalidBinaryOp { bin_op: char, lhs: Value, rhs: Value, reason: String }
}

pub enum Exception {
    UncheckedException(UncheckedException),
    CheckedException(Box<dyn 'static + Error + Send>)
}
