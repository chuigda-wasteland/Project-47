use std::error::Error;

use crate::data::Value;
use crate::data::wrapper::DynBase;

pub enum UncheckedException {
    ArgCountMismatch { func_ptr: usize, expected: usize, got: usize },
    InvalidBinaryOp { bin_op: char, lhs: Value, rhs: Value, reason: String }
}

#[cfg(feature = "async")]
pub type UserUncheckedException = Box<dyn Error + Send + 'static>;

#[cfg(not(feature = "async"))]
pub type UserUncheckedException = Box<dyn Error + 'static>;

pub type CheckedException = Box<dyn DynBase>;

pub enum Exception {
    UncheckedException(UncheckedException),
    UserUncheckedException(UserUncheckedException),
    CheckedException(CheckedException)
}
