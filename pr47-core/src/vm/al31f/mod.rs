use std::marker::PhantomData;

use crate::vm::al31f::alloc::Alloc;

pub mod alloc;
pub mod compiled;
pub mod insc;
pub mod stack;


pub struct AL31F<A: Alloc> {
    pub alloc: A
}

pub struct VMTask<A: Alloc> {
    _phantom: PhantomData<A>
}
