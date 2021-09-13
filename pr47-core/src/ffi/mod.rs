use std::any::TypeId;

use crate::data::tyck::TyckInfo;
use crate::util::either::Either;
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
    pub param_types: Box<[TyckInfo]>,
    pub param_options: Box<[DataOption]>,
    pub ret_type: Box<[TyckInfo]>,
    pub ret_option: Box<[DataOption]>,
    pub exceptions: Box<[TypeId]>
}

pub type FFIException = Either<CheckedException, UncheckedException>;
