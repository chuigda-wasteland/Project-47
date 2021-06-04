use std::any::TypeId;

use crate::data::tyck::TyckInfo;

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
    pub ret_type: TyckInfo,
    pub ret_option: DataOption,
    pub exceptions: Box<[TypeId]>
}
