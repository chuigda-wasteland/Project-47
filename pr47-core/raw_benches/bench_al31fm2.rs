use std::env;

use pr47::data::Value;
use pr47::data::exception::Exception;
use pr47::util::async_utils::block_on_future;
use pr47::vm::al31f::compiled::CompiledProgram;
use pr47::vm::al31f::VMThread;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::executor::{create_vm_main_thread, vm_thread_run_function};
use pr47::vm::test_program::{alloc_1m_program, fibonacci_program};

fn bench_fibonacci_call() {
    async fn run_fib35() {
        let program: CompiledProgram<DefaultAlloc> = fibonacci_program();

        for _ in 0..10 {
            let alloc: DefaultAlloc = DefaultAlloc::new();
            let mut vm_thread: Box<VMThread<DefaultAlloc>> =
                create_vm_main_thread(alloc, &program).await;

            let result: Result<Vec<Value>, Exception> = unsafe {
                vm_thread_run_function(&mut vm_thread, 0, &[Value::new_int(35)]).await
            };
            if let Err(_) = result {
                panic!("");
            }
        }
    }

    block_on_future(run_fib35())
}

fn bench_new_1m() {
    async fn run_new_1m() {
        let program: CompiledProgram<DefaultAlloc> = alloc_1m_program();
        for _ in 0..10 {
            let alloc: DefaultAlloc = DefaultAlloc::new();
            let mut vm_thread: Box<VMThread<DefaultAlloc>> =
                create_vm_main_thread(alloc, &program).await;

            let result: Result<Vec<Value>, Exception> = unsafe {
                vm_thread_run_function(&mut vm_thread, 0, &[]).await
            };
            if let Err(_) = result {
                panic!("");
            }
        }
    }

    block_on_future(run_new_1m())
}

fn main() {
    match env::args().collect::<Vec<_>>()[1].as_str() {
        "fib35" => bench_fibonacci_call(),
        "new1m" => bench_new_1m(),
        _ => panic!("Do you really know how to use this benchmarking suite? Don't make me laugh.")
    }
}
