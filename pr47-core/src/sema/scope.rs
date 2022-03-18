use std::collections::HashMap;
use std::ptr::NonNull;

use crate::data::tyck::TyckInfo;
use crate::sema::arena::ArenaPtr;
use crate::sema::decl::{FuncDecl, ObjectDecl};

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug))]
#[repr(u8)]
pub enum ScopeKind {
    Global,
    Function,
    Local
}

pub struct Scope<'s> {
    pub scope_kind: ScopeKind,
    pub parent: Option<Box<Scope<'s>>>,

    pub object_decls: HashMap<&'s str, ArenaPtr<'s, ObjectDecl<'s>>>,
    pub func_decls: HashMap<&'s str, Vec<ArenaPtr<'s, FuncDecl<'s>>>>,
    pub types: HashMap<&'s str, NonNull<TyckInfo>>
}

impl<'s> Scope<'s> {
    pub fn new(scope_kind: ScopeKind) -> Self {
        Scope {
            scope_kind,
            parent: None,

            object_decls: HashMap::new(),
            func_decls: HashMap::new(),
            types: HashMap::new()
        }
    }

    pub fn with_parent(scope_kind: ScopeKind, parent: Box<Scope<'s>>) -> Self {
        Scope {
            scope_kind,
            parent: Some(parent),

            object_decls: HashMap::new(),
            func_decls: HashMap::new(),
            types: HashMap::new()
        }
    }

    pub fn pop_self(self) -> Option<Box<Scope<'s>>> {
        self.parent
    }

    pub fn lookup_var_decl(&self, name: &str) -> Option<ArenaPtr<'s, ObjectDecl<'s>>> {
        if let Some(decl) = self.object_decls.get(name) {
            Some(*decl)
        } else if let Some(parent) = &self.parent {
            parent.lookup_var_decl(name)
        } else {
            None
        }
    }
}
