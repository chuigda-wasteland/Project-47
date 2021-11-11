pub mod attr;
pub mod decl;
pub mod expr;
pub mod id;
pub mod stmt;
pub mod token;
pub mod ty;

pub struct ConcreteProgram {
    pub global_attrs: Vec<attr::AttrList>,
    pub decls: Vec<decl::ConcreteDecl>
}

impl ConcreteProgram {
    pub fn new() -> Self {
        Self {
            global_attrs: vec![],
            decls: vec![]
        }
    }
}
