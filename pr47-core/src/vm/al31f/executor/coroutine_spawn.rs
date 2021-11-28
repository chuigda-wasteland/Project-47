use std::mem::transmute;
use std::ptr::NonNull;

use futures::TryFutureExt;
use tokio::task::{JoinError, JoinHandle};
use xjbutil::boxed_slice;
use xjbutil::unchecked::{UncheckedSendFut, UncheckedSendSync};

use crate::builtins::closure::Closure;
use crate::data::exception::UncheckedException;
use crate::data::Value;
use crate::ffi::async_fn::{AsyncReturnType, Promise, PromiseContext, PromiseGuard};
use crate::ffi::FFIException;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::CompiledProgram;
use crate::vm::al31f::executor::{vm_thread_run_function, VMThread};
use crate::vm::al31f::stack::{Stack, StackSlice};

#[inline(never)]
pub unsafe fn coroutine_spawn<A: Alloc>(
    thread: &mut VMThread<A>,
    slice: &mut StackSlice,
    func: &usize,
    args: &[usize]
) -> Promise<A> {
    let thread: &'static mut VMThread<A> = transmute::<_, _>(thread);

    let func: Value = slice.get_value(*func);
    let arg_values = args.iter().map(|arg: &usize| slice.get_value(*arg));

    let (func_id, args): (usize, Box<[Value]>) = if func.is_value() {
        let func_id: usize = func.vt_data.inner.int_value as usize;
        (func_id, arg_values.collect())
    } else {
        let closure: &Closure = &*(func.get_as_mut_ptr::<Closure>() as *const _);
        (closure.func_id, closure.captures.iter().map(|value: &Value| *value).chain(arg_values).collect())
    };

    let program: NonNull<CompiledProgram<A>> = thread.program;

    let arg_pack: UncheckedSendSync<_> = UncheckedSendSync::new((args, program));
    let get_join_handle = async move {
        let join_handle: JoinHandle<AsyncReturnType> = thread.vm.co_spawn_task(
            |child_context, (func_id, arg_pack)| UncheckedSendFut::new(async move {
                let (args, program): (Box<[Value]>, NonNull<CompiledProgram<A>>)
                    = arg_pack.into_inner();
                let mut new_thread: VMThread<A> = VMThread {
                    vm: child_context,
                    program,
                    stack: Stack::new()
                };

                let arg_pack = UncheckedSendSync::new((&mut new_thread, func_id, args.as_ref()));
                AsyncReturnType(match vm_thread_run_function::<_, false>(arg_pack) {
                    Ok(f) => f.await.into_inner().map_or_else(
                        |exception| Err(exception.inner),
                        |result| Ok(result.into_boxed_slice())
                    ),
                    Err(err) => Err(err.inner)
                })
            }), (func_id, arg_pack)
        ).await;

        let join_handle = join_handle.map_ok_or_else(
            |err: JoinError| AsyncReturnType(Err(
                FFIException::Unchecked(UncheckedException::JoinError {
                    inner: err
                })
            )),
            |data: AsyncReturnType| data
        );

        let join_handle: Promise<A> = Promise {
            fut: Box::pin(join_handle),
            ctx: PromiseContext {
                guard: PromiseGuard {
                    guards: boxed_slice![],
                    reset_guard_count: 0
                },
                resolver: None
            }
        };
        let join_handle_value: Value = Value::new_owned(join_handle);
        thread.vm.get_shared_data_mut().alloc.add_managed(join_handle_value.ptr_repr);
        AsyncReturnType(Ok(boxed_slice![join_handle_value]))
    };

    Promise {
        fut: Box::pin(get_join_handle),
        ctx: PromiseContext {
            guard: PromiseGuard {
                guards: boxed_slice![],
                reset_guard_count: 0
            },
            resolver: None
        }
    }
}
