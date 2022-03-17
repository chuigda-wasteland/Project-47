use std::hint::unreachable_unchecked;

use crate::sema::decl::{FuncDecl, ObjectDecl};

pub enum ASTNode<'s> {
    ObjectDeclNode(ObjectDecl<'s>),
    FuncDeclNode(FuncDecl<'s>)
}

pub unsafe trait DynCast<T> {
    unsafe fn dyn_cast(&self) -> &T;
    unsafe fn dyn_cast_mut(&mut self) -> &mut T;
}

macro_rules! impl_dyn_cast {
    ($e:ident, $t:ident) => {
        unsafe impl<'s> DynCast<$t<'s>> for ASTNode<'s> {
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
