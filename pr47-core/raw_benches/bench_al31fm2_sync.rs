use std::env;

use pr47::data::Value;
use pr47::data::exception::Exception;
use pr47::vm::al31f::alloc::default_alloc::DefaultAlloc;
use pr47::vm::al31f::compiled::CompiledProgram;
use pr47::vm::al31f::executor::vm_run_function_sync;
use pr47::vm::al31f::test_program::{alloc_1m_program, fibonacci_program};

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

const SUCK_WORDS: &'static str =
    "Do you really know how to use this benchmarking suite? Don't make me laugh.";

fn main() {
    match env::var("BENCH_ITEM").expect(SUCK_WORDS).to_lowercase().as_str() {
        "fib35" => bench_fibonacci_call(),
        "new1m" => bench_new_1m(),
        _ => panic!("{}", SUCK_WORDS)
    }
}
