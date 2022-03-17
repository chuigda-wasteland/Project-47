use std::ptr::NonNull;

use crate::data::tyck::TyckInfo;
use crate::sema::decl_context::DeclContext;
use crate::syntax::decl::{ConcreteFuncDecl, ConcreteObjectDecl};

pub struct ObjectDecl<'s> {
    pub name: &'s str,
    pub is_const: bool,
    pub ty: NonNull<TyckInfo>,

    pub concrete: &'s ConcreteObjectDecl<'s>
}

pub struct FuncDecl<'s> {
    pub name: &'s str,
    pub param_decl_context: DeclContext<'s>,
    pub ret_types: Vec<NonNull<TyckInfo>>,
    pub exception_spec: Vec<NonNull<TyckInfo>>,
    pub func_body: (), // TODO

    pub concrete: &'s ConcreteFuncDecl<'s>
}
