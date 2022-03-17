use std::collections::HashMap;
use std::ptr::NonNull;

use crate::data::tyck::TyckInfo;
use crate::sema::decl::{FuncDecl, ObjectDecl};

pub struct Scope<'s> {
    pub parent: Option<Box<Scope<'s>>>,

    pub object_decls: HashMap<&'s str, ObjectDecl<'s>>,
    pub func_decls: HashMap<&'s str, FuncDecl<'s>>,
    pub types: HashMap<&'s str, NonNull<TyckInfo>>
}
