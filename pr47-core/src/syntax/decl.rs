//! # Concrete syntax tree of declarations
//!
//! Declaration syntax:
//! ```text
//! declaration ::= maybe-attributed-declaration
//!               | import-declaration
//!               | export-declaration
//!
//! maybe-attributed-declaration ::= maybe-attribute attr-able-declaration
//!
//! maybe-attribute ::= attribute
//!                   | NIL
//!
//! attr-able-declaration ::= const-declaration
//!                         | func-declaration
//!                         | var-declaration
//!
//! import-declaration ::= 'import' identifier ';'
//!
//! open-import-declaration ::= 'open' 'import' identifier 'using' '(' identifier-list ')';
//!
//! export-declaration ::= 'export' '(' non-empty-identifier-list ')' ';'
//!
//! identifier-list ::= non-empty-identifier-list
//!                   | NIL
//!
//! non-empty-identifier-list ::= non-empty-identifier-list ',' identifier
//!                             | identifier
//! ```

use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::attr::Attribute;
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::id::Identifier;
use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::ty::ConcreteType;

pub enum ConcreteDecl<'a> {
    ConstDecl(ConcreteObjectDecl<'a>),
    ExportDecl(ConcreteExportDecl<'a>),
    FuncDecl(ConcreteFuncDecl<'a>),
    ImportDecl(ConcreteImportDecl<'a>),
    OpenImportDecl(ConcreteOpenImportDecl<'a>),
    VarDecl(ConcreteObjectDecl<'a>)
}

pub struct ConcreteObjectDecl<'a> {
    pub attr: Option<Attribute<'a>>,

    pub name: String,
    pub obj_type: Option<ConcreteType<'a>>,
    pub init_expr: Option<ConcreteExpr<'a>>,

    pub kwd_range: SourceRange,
    pub name_range: SourceRange
}

pub struct FunctionParam<'a> {
    pub attr: Option<Attribute<'a>>,

    pub param_name: String,
    pub param_type: Option<ConcreteType<'a>>,

    pub param_name_range: SourceRange
}

pub struct ConcreteFuncDecl<'a> {
    pub attr: Option<Attribute<'a>>,

    pub func_name: String,
    pub func_param_list: Vec<FunctionParam<'a>>,
    pub func_return_type: Option<ConcreteType<'a>>,
    pub func_body: Option<ConcreteCompoundStmt<'a>>,

    pub func_kwd_range: SourceRange,
    pub func_name_range: SourceRange,
    pub param_open_paren_loc: SourceLoc,
    pub param_close_paren_loc: SourceLoc
}

pub struct ConcreteImportDecl<'a> {
    pub import_path: Identifier<'a>,
    pub import_kwd_range: SourceRange
}

pub struct ConcreteOpenImportDecl<'a> {
    pub import_path: Identifier<'a>,
    pub open_kwd_range: SourceRange,
    pub import_kwd_range: SourceRange
}

pub struct ConcreteExportDecl<'a> {
    pub export_path: Identifier<'a>,
    pub export_kwd_range: SourceRange
}
