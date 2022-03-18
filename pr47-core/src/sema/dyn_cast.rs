use std::hint::unreachable_unchecked;

use crate::sema::decl::{FuncDecl, ObjectDecl};
use crate::sema::expr::{
    AsExpr,
    AwaitExpr,
    BinaryExpr,
    FieldRefExpr,
    IdRefExpr,
    LiteralExpr,
    SubscriptExpr,
    UnaryExpr
};

pub enum ASTNode<'s> {
    ObjectDeclNode(ObjectDecl<'s>),
    FuncDeclNode(FuncDecl<'s>),
    LiteralExprNode(LiteralExpr<'s>),
    IdRefExprNode(IdRefExpr<'s>),
    UnaryExprNode(UnaryExpr<'s>),
    BinaryExprNode(BinaryExpr<'s>),
    SubscriptExprNode(SubscriptExpr<'s>),
    FieldRefExprNode(FieldRefExpr<'s>),
    AwaitExprNode(AwaitExpr<'s>),
    AsExprNode(AsExpr<'s>)
}

pub unsafe trait DynCast<T> {
    fn cast_from(data: T) -> Self;
    unsafe fn dyn_cast(&self) -> &T;
    unsafe fn dyn_cast_mut(&mut self) -> &mut T;
}

macro_rules! impl_dyn_cast {
    ($e:ident, $t:ident) => {
        unsafe impl<'s> DynCast<$t<'s>> for ASTNode<'s> {
            #[inline(always)]
            fn cast_from(data: $t<'s>) -> Self {
                ASTNode::$e(data)
            }

            #[inline(always)]
            unsafe fn dyn_cast(&self) -> &$t<'s> {
                if let ASTNode::$e(obj) = self { obj } else { unreachable_unchecked() }
            }

            #[inline(always)]
            unsafe fn dyn_cast_mut(&mut self) -> &mut $t<'s> {
                if let ASTNode::$e(obj) = self { obj } else { unreachable_unchecked() }
            }
        }
    }
}

impl_dyn_cast!(ObjectDeclNode, ObjectDecl);
impl_dyn_cast!(FuncDeclNode, FuncDecl);
impl_dyn_cast!(LiteralExprNode, LiteralExpr);
impl_dyn_cast!(IdRefExprNode, IdRefExpr);
impl_dyn_cast!(UnaryExprNode, UnaryExpr);
impl_dyn_cast!(BinaryExprNode, BinaryExpr);
impl_dyn_cast!(SubscriptExprNode, SubscriptExpr);
impl_dyn_cast!(FieldRefExprNode, FieldRefExpr);
impl_dyn_cast!(AwaitExprNode, AwaitExpr);
impl_dyn_cast!(AsExprNode, AsExpr);
