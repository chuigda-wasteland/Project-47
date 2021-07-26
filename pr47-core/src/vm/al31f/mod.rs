pub mod alloc;
pub mod compiled;
pub mod insc;
pub mod stack;

use std::ptr::NonNull;

use crate::ffi::sync_fn::VMContext;
use crate::util::serializer::Serializer;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::CompiledProgram;
use crate::vm::al31f::stack::Stack;

#[cfg(feature = "async")]
use crate::ffi::async_fn::AsyncVMContext;

pub struct AL31F<A: Alloc> {
    pub alloc: A
}

pub struct VMThread<A: Alloc> {
    #[cfg(feature = "async")]
    vm: Serializer<AL31F<A>>,
    #[cfg(not(feature = "async"))]
    vm: AL31F<A>,

    program: NonNull<CompiledProgram>,
    stack: Stack,
}

pub struct Combustor<A: Alloc> {
    vm: NonNull<AL31F<A>>,
    program: NonNull<CompiledProgram>,
    stack: NonNull<Stack>
}

pub struct AsyncCombustor<A: Alloc> {
    vm_thread: NonNull<VMThread<A>>
}

unsafe impl<A: Alloc> Send for AsyncCombustor<A> {}

unsafe impl<A: Alloc> Sync for AsyncCombustor<A> {}

impl<A: Alloc> VMContext for Combustor<A> {}

impl<A: Alloc> AsyncVMContext for AsyncCombustor<A> {
    type SharedData = AL31F<A>;

    fn serializer(&self) -> &Serializer<Self::SharedData> {
        unsafe { &self.vm_thread.as_ref().vm }
    }
}
