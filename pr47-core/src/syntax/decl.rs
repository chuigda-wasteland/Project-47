use crate::syntax::attr::AttrList;
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::ty::ConcreteType;
use crate::util::location::{SourceLoc, SingleLineRange};

pub enum ConcreteDecl {
    VarDecl(ConcreteObjectDecl),
    ConstDecl(ConcreteObjectDecl),
    FuncDecl(ConcreteFuncDecl)
}

pub struct ConcreteObjectDecl {
    pub attrs: Option<AttrList>,

    pub name: String,
    pub obj_type: Option<ConcreteType>,
    pub init_expr: Option<ConcreteExpr>,

    pub kwd_range: SingleLineRange,
    pub name_range: SingleLineRange
}

pub struct FunctionParam {
    pub param_name: String,
    pub param_type: Option<ConcreteType>,

    pub param_name_range: SingleLineRange
}

pub struct ConcreteFuncDecl {
    pub attrs: Option<AttrList>,

    pub func_name: String,
    pub func_param_list: Vec<FunctionParam>,
    pub func_return_type: Option<ConcreteType>,
    pub func_body: Option<ConcreteCompoundStmt>,

    pub func_kwd_range: SingleLineRange,
    pub func_name_range: SingleLineRange,
    pub param_open_paren_loc: SourceLoc,
    pub param_close_paren_loc: SourceLoc
}
