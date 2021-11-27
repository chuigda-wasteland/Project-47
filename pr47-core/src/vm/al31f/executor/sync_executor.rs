use std::ptr::NonNull;

use xjbutil::unchecked::UncheckedSendSync;

use crate::data::Value;
use crate::data::exception::Exception;
use crate::vm::al31f::AL31F;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::CompiledProgram;
use crate::vm::al31f::stack::Stack;
use crate::vm::al31f::executor::{vm_thread_run_function, VMThread};

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
            stack: Stack::new()
        };
        vm_thread_run_function::<_, true>(UncheckedSendSync::new((&mut thread, func_id, args)))?
            .await
            .into_inner()
    });

    #[cfg(not(feature = "async"))]
    return pollster::block_on(async {
        let mut thread: VMThread<A> = VMThread {
            vm,
            program: NonNull::new_unchecked(program as *const _ as *mut _),
            stack: Stack::new()
        };
        vm_thread_run_function::<_, true>(UncheckedSendSync::new((&mut thread, func_id, args)))?
            .await
    });
}
