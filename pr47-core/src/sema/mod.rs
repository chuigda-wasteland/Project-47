pub mod arena;
pub mod decl;
pub mod dyn_cast;
pub mod scope;
pub mod visitor;

use crate::data::tyck::TyckInfoPool;
use crate::diag::DiagContext;

pub struct SemaPhase1<'d> {
    diag: &'d mut DiagContext
}

pub struct SemaPhase2<'d> {
    tyck_info_pool: TyckInfoPool,
    diag: &'d mut DiagContext
}
