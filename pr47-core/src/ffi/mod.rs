use std::any::TypeId;
use std::ptr::NonNull;

use xjbutil::either::Either;

use crate::data::tyck::TyckInfo;
use crate::data::exception::{CheckedException, UncheckedException};

pub mod sync_fn;

#[cfg(feature = "async")]
pub mod async_fn;

pub enum DataOption {
    Share,
    MutShare,
    Move,
    Copy,
    Raw,
    RawUntyped
}

pub struct Signature {
    pub param_types: Box<[NonNull<TyckInfo>]>,
    pub param_options: Box<[DataOption]>,
    pub ret_type: Box<[NonNull<TyckInfo>]>,
    pub ret_option: Box<[NonNull<TyckInfo>]>,
    pub exceptions: Box<[TypeId]>
}

pub type FFIException = Either<CheckedException, UncheckedException>;
