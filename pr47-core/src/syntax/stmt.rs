use crate::syntax::decl::CSTDecl;
use crate::syntax::expr::CSTExpr;

use crate::util::location::SourceLocation;

#[derive(Debug)]
pub enum CSTStmt {
    CompoundStmt(CSTCompoundStmt),
    ExprStmt(CSTExpr, SourceLocation),
    DeclStmt(CSTDecl, SourceLocation)
}

#[derive(Debug)]
pub struct CSTCompoundStmt {
    pub stmts: Vec<CSTStmt>,

    pub left_brace_loc: SourceLocation,
    pub right_brace_loc: SourceLocation
}
