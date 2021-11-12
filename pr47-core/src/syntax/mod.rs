pub mod attr;
pub mod decl;
pub mod expr;
pub mod id;
pub mod stmt;
pub mod token;
pub mod ty;

pub struct ConcreteProgram<'a> {
    pub global_attrs: Vec<attr::Attribute<'a>>,
    pub decls: Vec<decl::ConcreteDecl<'a>>
}

impl<'a> ConcreteProgram<'a> {
    pub fn new() -> Self {
        Self {
            global_attrs: vec![],
            decls: vec![]
        }
    }
}
