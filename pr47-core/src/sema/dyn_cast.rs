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

unsafe impl<'s> DynCast<ObjectDecl<'s>> for ASTNode<'s> {
    unsafe fn dyn_cast(&self) -> &ObjectDecl<'s> {
        if let ASTNode::ObjectDeclNode(obj) = self { obj } else { unreachable_unchecked() }
    }

    unsafe fn dyn_cast_mut(&mut self) -> &mut ObjectDecl<'s> {
        if let ASTNode::ObjectDeclNode(obj) = self { obj } else { unreachable_unchecked() }
    }
}

unsafe impl<'s> DynCast<FuncDecl<'s>> for ASTNode<'s> {
    unsafe fn dyn_cast(&self) -> &FuncDecl<'s> {
        if let ASTNode::FuncDeclNode(func) = self { func } else { unreachable_unchecked() }
    }

    unsafe fn dyn_cast_mut(&mut self) -> &mut FuncDecl<'s> {
        if let ASTNode::FuncDeclNode(func) = self { func } else { unreachable_unchecked() }
    }
}
