use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

use crate::data::Value;
use crate::data::tyck::TyckInfo;

#[cfg(feature = "async-tokio")] use tokio::task::JoinError;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AlreadyAwaited {
    promise: Value
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ArgCountMismatch {
    pub func_id: usize,
    pub expected: usize,
    pub got: usize
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DivideByZero();

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InvalidBinaryOp {
    pub lhs: Value,
    pub rhs: Value,
    pub bin_op: char
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InvalidCastOp {
    pub dest_type: NonNull<u8>,
    pub src: Value
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InvalidUnaryOp {
    pub operand: Value,
    pub unary_op: char
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct OwnershipCheckFailure {
    pub value: Value,
    pub expected_mask: u8
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TypeCheckFailure {
    pub value: Value,
    pub expected: NonNull<TyckInfo>
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct OverloadCallFailure {
    pub overload_table: usize
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct UnexpectedNull();

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IndexOutOfBounds {
    pub indexed: Value,
    pub index: i64
}

#[cfg(feature = "async-tokio")]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TokioJoinError {
    pub is_panic: bool
}

#[repr(C)]
pub union RawExceptionInner {
    pub uninit: (),

    pub already_awaited: AlreadyAwaited,
    pub arg_count_mismatch: ArgCountMismatch,
    pub divide_by_zero: DivideByZero,
    pub invalid_binary_op: InvalidBinaryOp,
    pub invalid_cast_op: InvalidCastOp,
    pub invalid_unary_op: InvalidUnaryOp,
    pub ownership_check_failure: OwnershipCheckFailure,
    pub type_check_failure: TypeCheckFailure,
    pub overload_call_failure: OverloadCallFailure,
    pub unexpected_null: UnexpectedNull,
    pub index_out_of_bounds: IndexOutOfBounds,
    #[cfg(feature = "async-tokio")]
    pub tokio_join_error: TokioJoinError,

    pub checked_exception: Value
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum RawExceptionTag {
    NoProblem = 0,

    AlreadyAwaited = 1,
    ArgCountMismatch = 2,
    DivideByZero = 3,
    InvalidBinaryOp = 4,
    InvalidCastOp = 5,
    InvalidUnaryOp = 6,
    OwnershipCheckFailure = 7,
    TypeCheckFailure = 8,
    OverloadCallFailure = 9,
    UnexpectedNull = 10,
    IndexOutOfBounds = 11,
    #[cfg(feature = "async-tokio")]
    TokioJoinError = 12,

    CheckedException = 13
}

#[repr(C)]
pub struct RawException {
    pub tag: RawExceptionTag,
    pub inner: RawExceptionInner
}

impl RawException {
    #[inline(always)]
    pub const fn is_ok(&self) -> bool {
        self.tag == RawExceptionTag::NoProblem
    }

    #[inline(always)]
    pub const fn is_checked(&self) -> bool {
        self.tag == RawExceptionTag::CheckedException
    }

    #[inline(always)]
    pub const fn already_awaited(promise: Value) -> Self {
        Self {
            tag: RawExceptionTag::AlreadyAwaited,
            inner: RawExceptionInner {
                already_awaited: AlreadyAwaited { promise }
            }
        }
    }

    #[cfg(feature = "async-tokio")]
    #[inline(always)]
    pub const fn join_error(join_error: JoinError) -> Self {
        Self {
            tag: RawExceptionTag::TokioJoinError,
            inner: RawExceptionInner {
                tokio_join_error: TokioJoinError {
                    is_panic: join_error.is_panic()
                }
            }
        }
    }
}

impl Debug for RawException {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawException({})", self.tag as u32)
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
