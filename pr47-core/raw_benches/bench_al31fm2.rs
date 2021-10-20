use std::env;

use xjbutil::async_utils::block_on_future;

use pr47::data::Value;
use pr47::data::exception::Exception;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::compiled::CompiledProgram;
use pr47::vm::al31f::executor::{VMThread, create_vm_main_thread, vm_thread_run_function};
use pr47::vm::al31f::test_program::{
    alloc_1m_program,
    bench_ffi_call_program,
    bench_ffi_call_program2,
    bench_raw_iter_program,
    fibonacci_program
};

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

fn bench_raw_iter() {
    let raw_iter_program: CompiledProgram<DefaultAlloc> = bench_raw_iter_program();
    eprintln!("raw iteration for 100,000,000 times: ");
    block_on_future(run_program(raw_iter_program, vec![]));
}

fn bench_ffi_call() {
    let raw_iter_program: CompiledProgram<DefaultAlloc> = bench_raw_iter_program();
    let program: CompiledProgram<DefaultAlloc> = bench_ffi_call_program();
    let program2: CompiledProgram<DefaultAlloc> = bench_ffi_call_program2();
    eprintln!("raw iteration for 100,000,000 times: ");
    block_on_future(run_program(raw_iter_program, vec![]));
    eprintln!("do FFI call for 100,000,000 times: ");
    block_on_future(run_program(program, vec![]));
    eprintln!("do FFI call for 10,000 * 10,000 times: ");
    block_on_future(run_program(program2, vec![]));
}

const SUCK_WORDS: &'static str =
    "Do you really know how to use this benchmarking suite? Don't make me laugh.";

fn main() {
    match env::var("BENCH_ITEM").expect(SUCK_WORDS).to_lowercase().as_str() {
        "fib35" => bench_fibonacci_call(),
        "new1m" => bench_new_1m(),
        "ffi" => bench_ffi_call(),
        "raw_iter" => bench_raw_iter(),
        _ => panic!("{}", SUCK_WORDS)
    }
}
