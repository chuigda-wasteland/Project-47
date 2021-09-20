use std::env;

use pr47::data::Value;
use pr47::data::exception::Exception;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::compiled::CompiledProgram;
use pr47::vm::al31f::executor::vm_run_function_sync;
use pr47::vm::al31f::test_program::{
    alloc_1m_program,
    bench_ffi_call_program,
    bench_ffi_call_program2,
    bench_raw_iter_program,
    fibonacci_program
};

fn run_program(mut program: CompiledProgram<DefaultAlloc>, args: Vec<Value>) {
    for _ in 0..10 {
        let alloc: DefaultAlloc = DefaultAlloc::new();
        let result: Result<Vec<Value>, Exception> = unsafe {
            vm_run_function_sync(alloc, &mut program, 0, &args)
        };
        if let Err(_) = result {
            panic!("");
        }
    }
}

fn bench_fibonacci_call() {
    let program: CompiledProgram<DefaultAlloc> = fibonacci_program();
    run_program(program, vec![Value::new_int(35)]);
}

fn bench_new_1m() {
    let program: CompiledProgram<DefaultAlloc> = alloc_1m_program();
    run_program(program, vec![]);
}

fn bench_raw_iter() {
    let raw_iter_program: CompiledProgram<DefaultAlloc> = bench_raw_iter_program();
    run_program(raw_iter_program, vec![]);
}

fn bench_ffi() {
    let raw_iter_program: CompiledProgram<DefaultAlloc> = bench_raw_iter_program();
    let program: CompiledProgram<DefaultAlloc> = bench_ffi_call_program();
    let program2: CompiledProgram<DefaultAlloc> = bench_ffi_call_program2();

    eprintln!("raw iteration for 100,000,000 times: ");
    run_program(raw_iter_program, vec![]);
    eprintln!("do FFI call for 100,000,000 times: ");
    run_program(program, vec![]);
    eprintln!("do FFI call for 10,000 * 10,000 times: ");
    run_program(program2, vec![]);
}

const SUCK_WORDS: &'static str =
    "Do you really know how to use this benchmarking suite? Don't make me laugh.";

fn main() {
    match env::var("BENCH_ITEM").expect(SUCK_WORDS).to_lowercase().as_str() {
        "fib35" => bench_fibonacci_call(),
        "new1m" => bench_new_1m(),
        "ffi" => bench_ffi(),
        "raw_iter" => bench_raw_iter(),
        _ => panic!("{}", SUCK_WORDS)
    }
}
