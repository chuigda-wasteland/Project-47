use std::ptr::NonNull;
use xjbutil::value::Value;

use crate::data::tyck::TyckInfo;
use crate::sema::arena::{Arena, ArenaPtr};
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
    SubscriptExpr(ArenaPtr<'s, BinaryExpr<'s>>),
    FieldRefExpr(ArenaPtr<'s, FieldRefExpr<'s>>),
    AwaitExpr(ArenaPtr<'s, AwaitExpr<'s>>),
    AsExpr(ArenaPtr<'s, AsExpr<'s>>)
}

impl<'s> Expr<'s> {
    pub fn get_const_fold_value<'a>(
        &self,
        arena: &'a Arena<'s>,
    ) -> Option<&'a Value> {
        (match &self {
            Expr::LiteralExpr(_) => return None,
            Expr::UnaryExpr(expr) => &expr.get_tricky(arena).maybe_constant_folding,
            Expr::BinaryExpr(expr) => &expr.get_tricky(arena).maybe_constant_folding,
            Expr::SubscriptExpr(expr) => &expr.get_tricky(arena).maybe_constant_folding,
            Expr::FieldRefExpr(expr) => &expr.get_tricky(arena).maybe_constant_folding,
            Expr::AwaitExpr(_) => return None,
            Expr::AsExpr(expr) => &expr.get_tricky(arena).maybe_constant_folding,
        }).as_ref()
    }

    pub fn get_type<'a>(
        &self,
        arena: &'a Arena<'s>
    ) -> Option<NonNull<TyckInfo>> {
        match self {
            Expr::LiteralExpr(expr) => Some(expr.get_tricky(arena).ty),
            Expr::UnaryExpr(expr) => expr.get_tricky(arena).ty,
            Expr::BinaryExpr(expr) => expr.get_tricky(arena).ty,
            Expr::SubscriptExpr(expr) => expr.get_tricky(arena).ty,
            Expr::FieldRefExpr(expr) => expr.get_tricky(arena).ty,
            Expr::AwaitExpr(expr) => Some(expr.get_tricky(arena).ty),
            Expr::AsExpr(expr) => Some(expr.get_tricky(arena).as_type)
        }
    }
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
    pub operand: Expr<'s>,
    pub ty: Option<NonNull<TyckInfo>>,

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
    pub lhs: Expr<'s>,
    pub rhs: Expr<'s>,
    pub ty: Option<NonNull<TyckInfo>>,

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
    pub base: Expr<'s>,
    pub index: Expr<'s>,
    pub ty: Option<NonNull<TyckInfo>>,
    pub tyck_base: bool,
    pub tyck_index: bool,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteSubscriptExpr<'s>
}

pub struct FieldRefExpr<'s> {
    pub base: Expr<'s>,
    pub field: &'s str,
    pub ty: Option<NonNull<TyckInfo>>,
    pub tyck_base: bool,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteFieldRefExpr<'s>
}

pub struct AwaitExpr<'s> {
    pub expr: Expr<'s>,
    pub ty: NonNull<TyckInfo>,
    pub tyck_expr: bool,

    pub concrete: &'s ConcreteAwaitExpr<'s>
}

pub struct AsExpr<'s> {
    pub expr: Expr<'s>,
    pub as_type: NonNull<TyckInfo>,

    pub maybe_constant_folding: Option<Value>,
    pub concrete: &'s ConcreteAsExpr<'s>
}
