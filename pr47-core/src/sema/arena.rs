use std::marker::PhantomData;

use xjbutil::typed_arena::{TypedArena, ArenaPtr};

use crate::sema::dyn_cast::{ASTNode, DynCast};

pub struct Arena<'s> {
    inner: TypedArena<ASTNode<'s>, 1024>
}

#[derive(Clone, Copy)]
pub struct Ptr<'s, T> {
    inner: ArenaPtr<ASTNode<'s>>,
    _phantom: PhantomData<T>
}

impl<'s, T> Ptr<'s, T>
    where T: 's,
          ASTNode<'s>: DynCast<T>
{
    pub fn get_tricky<'a>(&self, arena: &'a Arena<'s>) -> &'a T {
        let r: &'a ASTNode<'s> = self.inner.get_tricky(&arena.inner);
        unsafe { DynCast::dyn_cast(r) }
    }

    pub fn get_tricky_mut<'a>(&self, arena: &'a mut Arena<'s>) -> &'a mut T {
        let r: &'a mut ASTNode<'s> = self.inner.get_tricky_mut(&mut arena.inner);
        unsafe { DynCast::dyn_cast_mut(r) }
    }
}
