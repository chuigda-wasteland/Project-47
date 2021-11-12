use crate::diag::location::SourceLoc;

use crate::syntax::decl::ConcreteDecl;
use crate::syntax::expr::ConcreteExpr;

pub enum ConcreteStmt<'a> {
    CompoundStmt(ConcreteCompoundStmt<'a>),
    ExprStmt(ConcreteExpr<'a>, SourceLoc),
    DeclStmt(ConcreteDecl<'a>, SourceLoc)
}

pub struct ConcreteCompoundStmt<'a> {
    pub stmts: Vec<ConcreteStmt<'a>>,

    pub left_brace_loc: SourceLoc,
    pub right_brace_loc: SourceLoc
}
