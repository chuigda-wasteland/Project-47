use std::env;

use pr47::data::Value;
use pr47::data::exception::Exception;
use pr47::util::async_utils::block_on_future;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::compiled::CompiledProgram;
use pr47::vm::al31f::executor::{VMThread, create_vm_main_thread, vm_thread_run_function};
use pr47::vm::al31f::test_program::{alloc_1m_program, fibonacci_program};

async fn run_program(program: CompiledProgram<DefaultAlloc>, args: Vec<Value>) {
    for _ in 0..10 {
        let alloc: DefaultAlloc = DefaultAlloc::new();
        let mut vm_thread: Box<VMThread<DefaultAlloc>> =
            create_vm_main_thread(alloc, &program).await;

        let result: Result<Vec<Value>, Exception> = unsafe {
            vm_thread_run_function(&mut vm_thread, 0, &args).await
        };
        if let Err(_) = result {
            panic!("");
        }
    }
}

fn bench_fibonacci_call() {
    let program: CompiledProgram<DefaultAlloc> = fibonacci_program();
    block_on_future(run_program(program, vec![Value::new_int(35)]));
}

fn bench_new_1m() {
    let program: CompiledProgram<DefaultAlloc> = alloc_1m_program();
    block_on_future(run_program(program, vec![]));
}

const SUCK_WORDS: &'static str =
    "Do you really know how to use this benchmarking suite? Don't make me laugh.";

fn main() {
    match env::var("BENCH_ITEM").expect(SUCK_WORDS).to_lowercase().as_str() {
        "fib35" => bench_fibonacci_call(),
        "new1m" => bench_new_1m(),
        _ => panic!("{}", SUCK_WORDS)
    }
}
