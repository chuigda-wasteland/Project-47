use crate::sema::arena::ArenaPtr;
use crate::sema::decl::{FuncDecl, ObjectDecl};

pub struct DeclContext<'s> {
    pub object_decls: Vec<ArenaPtr<'s, ObjectDecl<'s>>>,
    pub func_decls: Vec<ArenaPtr<'s, FuncDecl<'s>>>
}
