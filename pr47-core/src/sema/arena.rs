use std::marker::PhantomData;

use xjbutil::typed_arena::{TypedArena, ArenaPtr as TypedArenaPtr};

use crate::sema::dyn_cast::{ASTNode, DynCast};

pub struct Arena<'s> {
    inner: TypedArena<ASTNode<'s>, 1024>
}

pub struct ArenaPtr<'s, T: 's> {
    inner: TypedArenaPtr<ASTNode<'s>>,
    _phantom: PhantomData<T>
}

impl<'s, T: 's> Clone for ArenaPtr<'s, T> {
    fn clone(&self) -> Self {
        ArenaPtr {
            inner: self.inner.clone(),
            _phantom: PhantomData
        }
    }
}

impl<'s, T: 's> Copy for ArenaPtr<'s, T> {}

impl<'s, T> ArenaPtr<'s, T>
    where T: 's,
          ASTNode<'s>: DynCast<T>
{
    pub fn new_in(data: T, arena: &Arena<'s>) -> Self {
        ArenaPtr {
            inner: arena.inner.make(DynCast::cast_from(data)),
            _phantom: PhantomData
        }
    }

    pub fn get<'a>(&self, arena: &'a Arena<'s>) -> &'a T {
        let r: &'a ASTNode<'s> = self.inner.get(&arena.inner);
        unsafe { r.dyn_cast() }
    }

    pub fn get_mut<'a>(&self, arena: &'a mut Arena<'s>) -> &'a mut T {
        let r: &'a mut ASTNode<'s> = self.inner.get_mut(&mut arena.inner);
        unsafe { r.dyn_cast_mut() }
    }

    #[inline(always)]
    pub(crate) fn get_tricky<'a>(&self, arena: &'a Arena<'s>) -> &'a T {
        let r: &'a ASTNode<'s> = self.inner.get_tricky(&arena.inner);
        unsafe { r.dyn_cast() }
    }

    #[inline(always)]
    pub(crate) fn get_tricky_mut<'a>(&self, arena: &'a mut Arena<'s>) -> &'a mut T {
        let r: &'a mut ASTNode<'s> = self.inner.get_tricky_mut(&mut arena.inner);
        unsafe { r.dyn_cast_mut() }
    }
}
