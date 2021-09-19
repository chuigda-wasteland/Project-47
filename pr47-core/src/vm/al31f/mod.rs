use std::ptr::NonNull;

#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncVMContext;
use crate::ffi::sync_fn::VMContext;
use crate::util::mem::FatPointer;
#[cfg(feature = "async")] use crate::util::serializer::Serializer;
use crate::vm::al31f::alloc::Alloc;

pub mod alloc;
pub mod compiled;
pub mod executor;
pub mod insc;
pub mod stack;
#[cfg(test)]
pub mod test;
#[cfg(any(test, feature = "bench"))]
pub mod test_program;

pub struct AL31F<A: Alloc> {
    pub alloc: A
}

impl<A: Alloc> AL31F<A> {
    pub fn new(alloc: A) -> Self {
        Self { alloc }
    }
}

pub struct Combustor<A: Alloc> {
    vm: NonNull<AL31F<A>>
}

impl<A: Alloc> Combustor<A> {
    pub fn new(vm: NonNull<AL31F<A>>) -> Self {
        Self { vm }
    }
}

impl<A: Alloc> VMContext for Combustor<A> {
    fn allocate(&mut self, fat_ptr: FatPointer) {
        unsafe { self.vm.as_mut().alloc.add_managed(fat_ptr); }
    }

    fn mark(&mut self, fat_ptr: FatPointer) {
        unsafe { self.vm.as_mut().alloc.mark_object(fat_ptr); }
    }
}

#[cfg(feature = "async")]
pub struct AsyncCombustor<A: Alloc> {
    vm: NonNull<Serializer<AL31F<A>>>
}

#[cfg(feature = "async")]
impl<A: Alloc> AsyncCombustor<A> {
    pub fn new(vm: NonNull<Serializer<AL31F<A>>>) -> Self {
        Self { vm }
    }
}

#[cfg(feature = "async")]
unsafe impl<A: Alloc> Send for AsyncCombustor<A> {}

#[cfg(feature = "async")]
unsafe impl<A: Alloc> Sync for AsyncCombustor<A> {}

#[cfg(feature = "async")]
impl<A: Alloc> AsyncVMContext for AsyncCombustor<A> {
    type SharedData = AL31F<A>;

    fn serializer(&self) -> &Serializer<Self::SharedData> {
        unsafe { self.vm.as_ref() }
    }
}
