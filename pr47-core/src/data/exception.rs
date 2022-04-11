use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

use crate::data::Value;
use crate::data::tyck::TyckInfo;

#[cfg(feature = "async-astd")] use std::convert::Infallible as JoinError;
#[cfg(feature = "async-tokio")] use tokio::task::JoinError;


pub enum UncheckedException {
    AlreadyAwaited { promise: Value },
    ArgCountMismatch { func_id: usize, expected: usize, got: usize },
    DivideByZero,
    InvalidBinaryOp { bin_op: char, lhs: Value, rhs: Value },
    InvalidCastOp { dest_type: &'static str, src: Value },
    InvalidUnaryOp { unary_op: char, src: Value },
    OwnershipCheckFailure { object: Value, expected_mask: u8 },
    TypeCheckFailure { object: Value, expected_type: NonNull<TyckInfo> },
    OverloadCallFailure { overload_table: usize },
    UnexpectedNull { value: Value },
    IndexOutOfBounds { indexed: Value, index: i64 },
    #[cfg(feature = "async")]
    JoinError { inner: JoinError }
}

pub type CheckedException = Value;

pub enum ExceptionInner {
    Unchecked(UncheckedException),
    Checked(CheckedException)
}

impl Debug for ExceptionInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO should we provide prettier printing?
        match self {
            ExceptionInner::Unchecked(_) => write!(f, "ExceptionInner::Unchecked"),
            ExceptionInner::Checked(_) => write!(f, "ExceptionInner::Checked")
        }
    }
}

#[derive(Clone, Copy)]
pub struct StackTrace {
    pub func_id: usize,
    pub insc_ptr: usize
}

impl StackTrace {
    pub fn new(func_id: usize, insc_ptr: usize) -> Self {
        Self { func_id, insc_ptr }
    }
}
