use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;
use crate::syntax::ty::ConcreteType;

pub enum ConcreteExpr<'a> {
    LiteralExpr(ConcreteLiteralExpr),
    StringLiteralExpr(ConcreteStringLiteralExpr<'a>),
    IdRefExpr(ConcreteIdRefExpr<'a>),
    UnaryExpr(ConcreteUnaryExpr<'a>),
    BinaryExpr(ConcreteBinaryExpr<'a>),
    AssignExpr(ConcreteAssignExpr<'a>),
    FuncCallExpr(ConcreteFuncCallExpr<'a>),
    SubscriptExpr(ConcreteSubscriptExpr<'a>),
    FieldRefExpr(ConcreteFieldRefExpr<'a>),
    MethodCallExpr(ConcreteMethodCallExpr<'a>),
    AsExpr(ConcreteAsExpr<'a>)
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

pub struct ConcreteIdRefExpr<'a> {
    pub id: Identifier<'a>
}

pub struct ConcreteStringLiteralExpr<'a> {
    pub value: &'a str,
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

pub struct ConcreteUnaryExpr<'a> {
    pub op: UnaryOp,
    pub operand: Box<ConcreteExpr<'a>>,

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

pub struct ConcreteBinaryExpr<'a> {
    pub op: BinaryOp,
    pub lhs: Box<ConcreteExpr<'a>>,
    pub rhs: Box<ConcreteExpr<'a>>,

    pub op_loc: SourceRange
}

pub struct ConcreteAssignExpr<'a> {
    pub lhs: Box<ConcreteExpr<'a>>,
    pub rhs: Box<ConcreteExpr<'a>>,

    pub op_loc: SourceLoc
}

pub struct ConcreteFuncCallExpr<'a> {
    pub func: Box<ConcreteExpr<'a>>,
    pub args: Vec<Box<ConcreteExpr<'a>>>,

    pub left_paren: SourceLoc,
    pub right_paren: SourceLoc
}

pub struct ConcreteSubscriptExpr<'a> {
    pub base: Box<ConcreteExpr<'a>>,
    pub idx: Box<ConcreteExpr<'a>>,

    pub left_bracket: SourceLoc,
    pub right_bracket: SourceLoc
}

pub struct ConcreteFieldRefExpr<'a> {
    pub base: Box<ConcreteExpr<'a>>,
    pub id: Identifier<'a>,

    pub dot_loc: SourceLoc
}

pub struct ConcreteMethodCallExpr<'a> {
    pub base: Box<ConcreteExpr<'a>>,
    pub func_id: Identifier<'a>,
    pub args: Vec<Box<ConcreteExpr<'a>>>,

    pub dot_loc: SourceLoc,
    pub left_paren: SourceLoc,
    pub right_paren: SourceLoc
}

pub struct ConcreteAsExpr<'a> {
    pub operand: Box<ConcreteExpr<'a>>,
    pub dest_type: ConcreteType<'a>,

    pub as_range: SourceRange
}
