use crate::diag::location::SourceLoc;

use crate::syntax::decl::ConcreteDecl;
use crate::syntax::expr::ConcreteExpr;

pub enum ConcreteStmt {
    CompoundStmt(ConcreteCompoundStmt),
    ExprStmt(ConcreteExpr, SourceLoc),
    DeclStmt(ConcreteDecl, SourceLoc)
}

pub struct ConcreteCompoundStmt {
    pub stmts: Vec<ConcreteStmt>,

    pub left_brace_loc: SourceLoc,
    pub right_brace_loc: SourceLoc
}
