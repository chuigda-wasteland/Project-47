pub mod alloc;
pub mod compiled;
pub mod executor;
pub mod insc;
pub mod stack;

#[cfg(test)] pub mod test;
#[cfg(any(test, feature = "bench"))] pub mod test_program;

use std::ptr::NonNull;

use xjbutil::wide_ptr::WidePointer;

use crate::ffi::sync_fn::VMContext;
use crate::vm::al31f::alloc::Alloc;

#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncVMContext;
#[cfg(feature = "async")] use crate::util::serializer::Serializer;

pub struct AL31F<A: Alloc> {
    pub alloc: A
}

impl<A: Alloc> AL31F<A> {
    pub fn new(alloc: A) -> Self {
        Self { alloc }
    }
}

#[cfg(not(feature = "async"))]
pub struct Combustor<A: Alloc> {
    vm: NonNull<AL31F<A>>
}

#[cfg(not(feature = "async"))]
impl<A: Alloc> Combustor<A> {
    pub fn new(vm: NonNull<AL31F<A>>) -> Self {
        Self { vm }
    }
}

#[cfg(not(feature = "async"))]
impl<A: Alloc> VMContext for Combustor<A> {
    fn allocate(&mut self, wide_ptr: WidePointer) {
        unsafe { self.vm.as_mut().alloc.add_managed(wide_ptr); }
    }

    fn mark(&mut self, wide_ptr: WidePointer) {
        unsafe { self.vm.as_mut().alloc.mark_object(wide_ptr); }
    }
}

#[cfg(feature = "async")]
pub struct Combustor<A: Alloc> {
    vm: NonNull<Serializer<AL31F<A>>>
}

#[cfg(feature = "async")]
impl<A: Alloc> Combustor<A> {
    pub fn new(vm: NonNull<Serializer<AL31F<A>>>) -> Self {
        Self { vm }
    }
}

#[cfg(feature = "async")]
unsafe impl<A: Alloc> Send for Combustor<A> {}

#[cfg(feature = "async")]
unsafe impl<A: Alloc> Sync for Combustor<A> {}

#[cfg(feature = "async")]
impl<A: Alloc> VMContext for Combustor<A> {
    fn allocate(&mut self, wide_ptr: WidePointer) {
        unsafe { self.vm.as_mut().get_shared_data_mut().alloc.add_managed(wide_ptr); }
    }

    fn mark(&mut self, wide_ptr: WidePointer) {
        unsafe { self.vm.as_mut().get_shared_data_mut().alloc.mark_object(wide_ptr); }
    }
}

#[cfg(feature = "async")]
impl<A: Alloc> AsyncVMContext for Combustor<A> {
    type SharedData = AL31F<A>;

    fn serializer(&self) -> &Serializer<Self::SharedData> {
        unsafe { self.vm.as_ref() }
    }
}
