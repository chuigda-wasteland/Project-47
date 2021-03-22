use crate::syntax::ty::ConcreteType;
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::attr::AttrList;

use crate::util::location::{SourceLocation, SingleLineRange};
use crate::util::mstring::StringHandle;

#[derive(Debug)]
pub enum ConcreteDecl {
    VarDecl(ConcreteVarDecl),
    FuncDecl(ConcreteFuncDecl)
}

#[derive(Debug)]
pub struct ConcreteVarDecl {
    pub attrs: Option<AttrList>,

    pub var_name: StringHandle,
    pub var_type: Option<ConcreteType>,
    pub init_expr: Option<ConcreteExpr>,

    pub var_kwd_range: SingleLineRange,
    pub var_name_range: SingleLineRange
}

#[derive(Debug)]
pub struct FunctionParam {
    pub param_name: StringHandle,
    pub param_type: Option<ConcreteType>,

    pub param_name_range: SingleLineRange
}

#[derive(Debug)]
pub struct ConcreteFuncDecl {
    pub attrs: Option<AttrList>,

    pub func_name: StringHandle,
    pub func_param_list: Vec<FunctionParam>,
    pub func_return_type: Option<ConcreteType>,
    pub func_body: Option<ConcreteCompoundStmt>,

    pub func_kwd_range: SingleLineRange,
    pub func_name_range: SingleLineRange,
    pub param_open_paren_loc: SourceLocation,
    pub param_close_paren_loc: SourceLocation
}
