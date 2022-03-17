use std::ptr::NonNull;

use xjbutil::typed_arena::ArenaPtr;

use crate::data::tyck::TyckInfo;
use crate::syntax::decl::{ConcreteFuncDecl, ConcreteObjectDecl};

pub struct ObjectDecl<'s> {
    pub name: &'s str,
    pub is_const: bool,
    pub ty: NonNull<TyckInfo>,

    pub concrete: &'s ConcreteObjectDecl<'s>
}

pub struct FuncDecl<'s> {
    pub name: &'s str,
    pub param_decls: Vec<ArenaPtr<ObjectDecl<'s>>>,
    pub ret_types: Vec<NonNull<TyckInfo>>,
    pub exception_spec: Vec<NonNull<TyckInfo>>,
    pub func_body: (), // TODO

    pub concrete: &'s ConcreteFuncDecl<'s>
}
