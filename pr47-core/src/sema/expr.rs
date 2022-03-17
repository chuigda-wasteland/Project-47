use std::ptr::NonNull;
use xjbutil::value::Value;

use crate::data::tyck::TyckInfo;
use crate::sema::arena::ArenaPtr;
use crate::syntax::expr::{
    ConcreteAsExpr,
    ConcreteAwaitExpr,
    ConcreteBinaryExpr,
    ConcreteFieldRefExpr,
    ConcreteLiteralExpr,
    ConcreteSubscriptExpr,
    ConcreteUnaryExpr,
    LiteralExprContent
};

pub enum Expr<'s> {
    LiteralExpr(ArenaPtr<'s, LiteralExpr<'s>>),
    UnaryExpr(ArenaPtr<'s, UnaryExpr<'s>>),
    BinaryExpr(ArenaPtr<'s, BinaryExpr<'s>>),
}

pub struct LiteralExpr<'s> {
    pub content: LiteralExprContent<'s>,
    pub ty: NonNull<TyckInfo>,

    pub concrete: &'s ConcreteLiteralExpr<'s>
}

#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ResolvedUnaryOp {
    Positive,
    Negation,

    IntNegation,
    FloatNegation,
    LogicalNot,
    BitwiseReverse
}

pub struct UnaryExpr<'s> {
    pub op: ResolvedUnaryOp,
    pub operand: ArenaPtr<'s, Expr<'s>>,
    pub ty: NonNull<TyckInfo>,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteUnaryExpr<'s>
}

#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ResolvedBinaryOp {
    Mul,
    IntMul,
    FloatMul,

    Div,
    IntDiv,
    FloatDiv,

    IntMod,
    IntModRTTI,

    Add,
    IntAdd,
    FloatAdd,
    StringAdd,

    Minus,
    IntMinus,
    FloatMinus,

    GreaterThan,
    IntGreaterThan,
    FloatGreaterThan,

    LessThan,
    IntLessThan,
    FloatLessThan,

    GreaterThanOrEqual,
    IntGreaterThanOrEqual,
    FloatGreaterThanOrEqual,

    LessThanOrEqual,
    IntLessThanOrEqual,
    FloatLessThanOrEqual,

    Equal,
    NotEqual,

    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAndRTTI,
    BitwiseOrRTTI,
    BitwiseXorRTTI,

    LogicalAnd,
    LogicalOr,
    LogicalXor,
    LogicalAndRTTI,
    LogicalOrRTTI,
    LogicalXorRTTI
}

pub struct BinaryExpr<'s> {
    pub op: ResolvedBinaryOp,
    pub lhs: ArenaPtr<'s, Expr<'s>>,
    pub rhs: ArenaPtr<'s, Expr<'s>>,
    pub ty: NonNull<TyckInfo>,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteBinaryExpr<'s>
}

#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SubscriptMode {
    ArrayIndex,
    ObjectIndex,
    Undetermined
}

pub struct SubscriptExpr<'s> {
    pub mode: SubscriptMode,
    pub base: ArenaPtr<'s, Expr<'s>>,
    pub index: ArenaPtr<'s, Expr<'s>>,
    pub ty: NonNull<TyckInfo>,
    pub tyck_base: bool,
    pub tyck_index: bool,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteSubscriptExpr<'s>
}

pub struct FieldRefExpr<'s> {
    pub base: ArenaPtr<'s, Expr<'s>>,
    pub field: &'s str,
    pub ty: NonNull<TyckInfo>,
    pub tyck_base: bool,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteFieldRefExpr<'s>
}

pub struct AwaitExpr<'s> {
    pub expr: ArenaPtr<'s, Expr<'s>>,
    pub ty: NonNull<TyckInfo>,
    pub tyck_expr: bool,

    pub concrete: &'s ConcreteAwaitExpr<'s>
}

pub struct AsExpr<'s> {
    pub expr: ArenaPtr<'s, Expr<'s>>,
    pub as_type: NonNull<TyckInfo>,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteAsExpr<'s>
}
