pub mod attr;
pub mod decl;
pub mod expr;
pub mod id;
pub mod stmt;
pub mod token;
pub mod ty;

pub struct ConcreteProgram {
    pub global_attrs: Vec<attr::Attribute>,
    pub decls: Vec<decl::ConcreteDecl>
}
