use crate::data::Value;

pub enum UncheckedException {
    ArgCountMismatch { func_ptr: usize, expected: usize, got: usize },
    InvalidBinaryOp { bin_op: char, lhs: Value, rhs: Value, reason: String }
}

pub type CheckedException = Value;

pub enum Exception {
    UncheckedException(UncheckedException),
    CheckedException(CheckedException)
}
