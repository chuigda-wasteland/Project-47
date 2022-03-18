pub mod arena;
pub mod decl;
pub mod decl_context;
pub mod expr;
pub mod dyn_cast;
pub mod scope;
pub mod visitor;

use crate::data::tyck::TyckInfoPool;
use crate::diag::DiagContext;

pub struct SemaPhase1<'d> {
    diag: &'d mut DiagContext
}

pub struct SemaPhase2<'s, 'd> {
    tyck_info_pool: &'s TyckInfoPool,
    diag: &'d mut DiagContext
}
