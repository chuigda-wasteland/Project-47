use std::marker::PhantomPinned;
use std::ptr::NonNull;

use xjbutil::unchecked::UncheckedSendSync;

use crate::data::Value;
use crate::vm::al31fm2::AL31F;
use crate::vm::al31fm2::alloc::Alloc;
use crate::vm::al31fm2::compiled::CompiledProgram;
use crate::vm::al31fm2::exception::Exception;
use crate::vm::al31fm2::executor::{VMThread, vm_thread_run_function};
use crate::vm::al31fm2::stack::Stack;

#[cfg(feature = "async")]
use crate::util::serializer::CoroutineContext;

pub unsafe fn vm_run_function_sync<A: Alloc>(
    alloc: A,
    program: &CompiledProgram<A>,
    func_id: usize,
    args: &[Value]
) -> Result<Vec<Value>, Exception> {
    let vm: AL31F<A> = AL31F::new(alloc);

    #[cfg(feature = "async")]
    return pollster::block_on(async {
        let vm: CoroutineContext<AL31F<A>> = CoroutineContext::main_context(vm).await;
        let mut thread: VMThread<A> = VMThread {
            vm,
            program: NonNull::new_unchecked(program as *const _ as *mut _),
            stack: Stack::new(),
            _phantom: PhantomPinned::default()
        };
        thread.vm.get_shared_data_mut().alloc.add_stack(&thread.stack);
        vm_thread_run_function::<_, true>(UncheckedSendSync::new((&mut thread, func_id, args)))?
            .await
            .into_inner()
    });

    #[cfg(not(feature = "async"))]
    return pollster::block_on(async {
        let mut thread: VMThread<A> = VMThread {
            vm,
            program: NonNull::new_unchecked(program as *const _ as *mut _),
            stack: Stack::new(),
            _phantom: PhantomPinned
        };
        thread.vm.alloc.add_stack(&thread.stack);
        vm_thread_run_function::<_, true>(UncheckedSendSync::new((&mut thread, func_id, args)))?
            .await
            .into_inner()
    });
}
