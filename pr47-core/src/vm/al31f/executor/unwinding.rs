use std::any::TypeId;

use unchecked_unwrap::UncheckedUnwrap;

use crate::data::exception::{CheckedException, Exception, UncheckedException};
use crate::data::Value;
use crate::vm::al31f::AL31F;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::stack::{FrameInfo, Stack, StackSlice};

#[inline(never)]
pub unsafe fn unchecked_exception_unwind_stack(
    unchecked_exception: UncheckedException,
    stack: &mut Stack,
    insc_ptr: usize
) -> Exception {
    let mut exception: Exception = Exception::unchecked_exc(unchecked_exception);

    let mut insc_ptr: usize = insc_ptr;
    while stack.frames.len() != 0 {
        let last_frame: &FrameInfo = stack.frames.last().unchecked_unwrap();
        exception.push_stack_trace(last_frame.func_id, insc_ptr);
        insc_ptr = last_frame.ret_addr.saturating_sub(1);

        stack.unwind_shrink_slice();
    }
    exception
}

#[inline(never)]
pub unsafe fn checked_exception_unwind_stack<A: Alloc>(
    vm: &mut AL31F<A>,
    program: &CompiledProgram<A>,
    checked_exception: CheckedException,
    stack: &mut Stack,
    insc_ptr: usize
) -> Result<(StackSlice, usize), Exception> {
    let exception_type_id: TypeId = (*checked_exception.get_as_dyn_base()).dyn_type_id();

    let mut exception: Exception = Exception::checked_exc(checked_exception);
    let mut insc_ptr: usize = insc_ptr;

    while stack.frames.len() != 0 {
        let frame: &FrameInfo = stack.frames.last().unchecked_unwrap();
        let func_id: usize = frame.func_id;
        exception.push_stack_trace(func_id, insc_ptr);

        let compiled_function: &CompiledFunction = &program.functions[func_id];

        if let Some(exc_handlers /*: &Box<[ExceptionHandlingBlock]>*/)
        = &compiled_function.exc_handlers
        {
            for exc_handler /*: &ExceptionHandlingBlock*/ in exc_handlers.as_ref().iter() {
                let (start_insc, end_insc): (usize, usize) = exc_handler.insc_ptr_range;
                if insc_ptr >= start_insc &&
                    insc_ptr <= end_insc &&
                    exception_type_id == exc_handler.exception_id
                {
                    let frame_size: usize = frame.frame_end - frame.frame_start;
                    let exception_value: Value = Value::new_owned(exception);
                    vm.alloc.add_managed(exception_value);
                    let mut stack_slice: StackSlice = stack.last_frame_slice();
                    stack_slice.set_value(frame_size - 1, exception_value);

                    return Ok((stack_slice, exc_handler.handler_addr));
                }
            }
        }

        let frame_ret_addr: usize = frame.ret_addr;
        insc_ptr = frame_ret_addr.saturating_sub(1);

        stack.unwind_shrink_slice();
    }

    Err(exception)
}
