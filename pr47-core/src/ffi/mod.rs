use std::any::TypeId;
use std::ptr::NonNull;

use xjbutil::either::Either;

use crate::data::tyck::TyckInfo;
use crate::data::exception::{CheckedException, UncheckedException};

pub mod sync_fn;

#[cfg(feature = "async")]
pub mod async_fn;

#[repr(u8)]
pub enum DataOption {
    Share      = 0b00000000,
    MutShare   = 0b00000001,
    Move       = 0b00000010,
    Copy       = 0b00000011,
    Raw        = 0b00000100,
    RawUntyped = 0b00000101
}

pub const DATA_OPTION_NULLABLE_MASK: u8 = 0b10000000;

pub struct Signature {
    pub param_types: Box<[NonNull<TyckInfo>]>,
    pub param_options: Box<[u8]>,
    pub ret_type: Box<[NonNull<TyckInfo>]>,
    pub ret_option: Box<[u8]>,
    pub exceptions: Box<[TypeId]>
}

pub type FFIException = Either<CheckedException, UncheckedException>;
