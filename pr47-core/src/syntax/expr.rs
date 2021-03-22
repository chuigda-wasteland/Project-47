use crate::syntax::id::Identifier;
use crate::util::location::{SingleLineRange, SourceLocation};
use crate::syntax::ty::CSTType;

#[derive(Debug)]
pub enum CSTExpr {
    LiteralExpr(CSTLiteralExpr),
    StringLiteralExpr(CSTStringLiteralExpr),
    IdRefExpr(CSTIdRefExpr),
    UnaryExpr(CSTUnaryExpr),
    BinaryExpr(CSTBinaryExpr),
    AssignExpr(CSTAssignExpr),
    FuncCallExpr(CSTFuncCallExpr),
    SubscriptExpr(CSTSubscriptExpr),
    FieldRefExpr(CSTFieldRefExpr),
    MethodCallExpr(CSTMethodCallExpr),
    AsExpr(CSTAsExpr)
}

#[derive(Debug)]
pub struct CSTLiteralExpr {
    pub content: LiteralExprContent,
    pub range: SingleLineRange
}

#[derive(Debug)]
pub enum LiteralExprContent {
    Byte(u8),
    Int(i64),
    Float(f64),
    Char(char),
    Boolean(bool)
}

#[derive(Debug)]
pub struct CSTIdRefExpr {
    pub id: Identifier
}

#[derive(Debug)]
pub struct CSTStringLiteralExpr {
    pub value: String,
    pub range: SingleLineRange
}

#[derive(Debug)]
pub enum UnaryOp {
    BitNot,
    Not,
    Negate
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        use UnaryOp::*;
        match self {
            BitNot => "~",
            Not => "!",
            Negate => "-"
        }
    }
}

#[derive(Debug)]
pub struct CSTUnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<CSTExpr>,

    pub op_loc: SourceLocation
}

#[derive(Debug)]
pub enum BinaryOp {
    BitAnd, BitOr, BitXor,
    Add, Sub,
    Mul, Div, Mod,
    Lt, Gt, Eq, LEq, GEq, NEq,
    And, Or, Xor
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        use BinaryOp::*;
        match self {
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Lt => "<",
            Gt => ">",
            Eq => "==",
            LEq => "<=",
            GEq => ">=",
            NEq => "!=",
            And => "&&",
            Or => "||",
            Xor => "^^"
        }
    }
}

#[derive(Debug)]
pub struct CSTBinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<CSTExpr>,
    pub rhs: Box<CSTExpr>,

    pub op_loc: SingleLineRange
}

#[derive(Debug)]
pub struct CSTAssignExpr {
    pub lhs: Box<CSTExpr>,
    pub rhs: Box<CSTExpr>,

    pub op_loc: SourceLocation
}

#[derive(Debug)]
pub struct CSTFuncCallExpr {
    pub func: Box<CSTExpr>,
    pub args: Vec<Box<CSTExpr>>,

    pub left_paren: SourceLocation,
    pub right_paren: SourceLocation
}

#[derive(Debug)]
pub struct CSTSubscriptExpr {
    pub base: Box<CSTExpr>,
    pub idx: Box<CSTExpr>,

    pub left_bracket: SourceLocation,
    pub right_bracket: SourceLocation
}

#[derive(Debug)]
pub struct CSTFieldRefExpr {
    pub base: Box<CSTExpr>,
    pub id: Identifier,

    pub dot_loc: SourceLocation
}

#[derive(Debug)]
pub struct CSTMethodCallExpr {
    pub base: Box<CSTExpr>,
    pub func_id: Identifier,
    pub args: Vec<Box<CSTExpr>>,

    pub dot_loc: SourceLocation,
    pub left_paren: SourceLocation,
    pub right_paren: SourceLocation
}

#[derive(Debug)]
pub struct CSTAsExpr {
    pub operand: Box<CSTExpr>,
    pub dest_type: CSTType,

    pub as_range: SingleLineRange
}
