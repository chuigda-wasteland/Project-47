use crate::syntax::decl::ConcreteDecl;
use crate::syntax::expr::ConcreteExpr;

use crate::util::location::SourceLocation;

pub enum ConcreteStmt {
    CompoundStmt(ConcreteCompoundStmt),
    ExprStmt(ConcreteExpr, SourceLocation),
    DeclStmt(ConcreteDecl, SourceLocation)
}

pub struct ConcreteCompoundStmt {
    pub stmts: Vec<ConcreteStmt>,

    pub left_brace_loc: SourceLocation,
    pub right_brace_loc: SourceLocation
}
