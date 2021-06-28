use std::marker::PhantomData;

use crate::vm::al31f::alloc::Alloc;

pub mod alloc;
pub mod stack;
pub mod insc;

pub struct AL31F<A: Alloc> {
    // TODO don't know what kind of structure is really required
    _phantom: PhantomData<A>
}
