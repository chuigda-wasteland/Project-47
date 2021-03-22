use crate::syntax::ty::CSTType;
use crate::syntax::expr::CSTExpr;
use crate::syntax::stmt::CSTCompoundStmt;
use crate::syntax::attr::AttrList;

use crate::util::location::{SourceLocation, SingleLineRange};
use crate::util::mstring::StringHandle;

#[derive(Debug)]
pub enum CSTDecl {
    VarDecl(CSTVarDecl),
    FuncDecl(CSTFuncDecl)
}

#[derive(Debug)]
pub struct CSTVarDecl {
    pub attrs: Option<AttrList>,

    pub var_name: StringHandle,
    pub var_type: Option<CSTType>,
    pub init_expr: Option<CSTExpr>,

    pub var_kwd_range: SingleLineRange,
    pub var_name_range: SingleLineRange
}

#[derive(Debug)]
pub struct FunctionParam {
    pub param_name: StringHandle,
    pub param_type: Option<CSTType>,

    pub param_name_range: SingleLineRange
}

#[derive(Debug)]
pub struct CSTFuncDecl {
    pub attrs: Option<AttrList>,

    pub func_name: StringHandle,
    pub func_param_list: Vec<FunctionParam>,
    pub func_return_type: Option<CSTType>,
    pub func_body: Option<CSTCompoundStmt>,

    pub func_kwd_range: SingleLineRange,
    pub func_name_range: SingleLineRange,
    pub param_open_paren_loc: SourceLocation,
    pub param_close_paren_loc: SourceLocation
}
