use std::ptr::NonNull;

use xjbutil::typed_arena::ArenaPtr;

use crate::data::tyck::TyckInfo;
use crate::syntax::decl::ConcreteDecl;

pub struct ObjectDecl<'s> {
    pub name: &'s str,
    pub is_const: bool,
    pub ty: NonNull<TyckInfo>,

    pub concrete: &'s ConcreteDecl<'s>
}

pub struct FuncDecl<'s> {
    pub name: &'s str,
    pub param_decls: Vec<ArenaPtr<ObjectDecl<'s>>>
}
