use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;
use crate::syntax::ty::ConcreteType;

pub enum ConcreteExpr {
    LiteralExpr(ConcreteLiteralExpr),
    StringLiteralExpr(ConcreteStringLiteralExpr),
    IdRefExpr(ConcreteIdRefExpr),
    UnaryExpr(ConcreteUnaryExpr),
    BinaryExpr(ConcreteBinaryExpr),
    AssignExpr(ConcreteAssignExpr),
    FuncCallExpr(ConcreteFuncCallExpr),
    SubscriptExpr(ConcreteSubscriptExpr),
    FieldRefExpr(ConcreteFieldRefExpr),
    MethodCallExpr(ConcreteMethodCallExpr),
    AsExpr(ConcreteAsExpr)
}

pub struct ConcreteLiteralExpr {
    pub content: LiteralExprContent,
    pub range: SourceRange
}

pub enum LiteralExprContent {
    Byte(u8),
    Int(i64),
    Float(f64),
    Char(char),
    Boolean(bool)
}

pub struct ConcreteIdRefExpr {
    pub id: Identifier
}

pub struct ConcreteStringLiteralExpr {
    pub value: String,
    pub range: SourceRange
}

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

pub struct ConcreteUnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<ConcreteExpr>,

    pub op_loc: SourceLoc
}

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

pub struct ConcreteBinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<ConcreteExpr>,
    pub rhs: Box<ConcreteExpr>,

    pub op_loc: SourceRange
}

pub struct ConcreteAssignExpr {
    pub lhs: Box<ConcreteExpr>,
    pub rhs: Box<ConcreteExpr>,

    pub op_loc: SourceLoc
}

pub struct ConcreteFuncCallExpr {
    pub func: Box<ConcreteExpr>,
    pub args: Vec<Box<ConcreteExpr>>,

    pub left_paren: SourceLoc,
    pub right_paren: SourceLoc
}

pub struct ConcreteSubscriptExpr {
    pub base: Box<ConcreteExpr>,
    pub idx: Box<ConcreteExpr>,

    pub left_bracket: SourceLoc,
    pub right_bracket: SourceLoc
}

pub struct ConcreteFieldRefExpr {
    pub base: Box<ConcreteExpr>,
    pub id: Identifier,

    pub dot_loc: SourceLoc
}

pub struct ConcreteMethodCallExpr {
    pub base: Box<ConcreteExpr>,
    pub func_id: Identifier,
    pub args: Vec<Box<ConcreteExpr>>,

    pub dot_loc: SourceLoc,
    pub left_paren: SourceLoc,
    pub right_paren: SourceLoc
}

pub struct ConcreteAsExpr {
    pub operand: Box<ConcreteExpr>,
    pub dest_type: ConcreteType,

    pub as_range: SourceRange
}
