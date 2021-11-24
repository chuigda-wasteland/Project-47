use std::ptr::NonNull;

use xjbutil::either::Either;

use crate::data::tyck::TyckInfo;
use crate::data::exception::{CheckedException, UncheckedException};

pub mod sync_fn;

#[cfg(feature = "async")]
pub mod async_fn;

#[repr(u8)]
pub enum DataOption {
    Share,
    MutShare,
    Move,
    Copy,
    Raw,
    RawUntyped
}

pub struct Signature {
    pub func_type: NonNull<TyckInfo>,

    pub param_options: Box<[DataOption]>,
    pub ret_option: Box<[DataOption]>
}

pub type FFIException = Either<CheckedException, UncheckedException>;
