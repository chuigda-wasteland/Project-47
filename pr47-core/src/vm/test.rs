use crate::boxed_slice as bslice;
use crate::data::Value;
use crate::data::exception::Exception;
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};
use crate::util::async_utils::block_on_future;
use crate::vm::al31f::VMThread;
use crate::vm::al31f::alloc::default_alloc::DefaultAlloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::executor::{create_vm_main_thread, vm_thread_run_function};
use crate::vm::al31f::insc::Insc;
use crate::vm::test_program::fibonacci_program;

async fn basic_program_eval() {
    let program: CompiledProgram<DefaultAlloc> = CompiledProgram {
        code: bslice![
            Insc::AddInt(0, 1, 0),
            Insc::Return(bslice![0])
        ],
        const_pool: bslice![],
        init_proc: 0,
        functions: bslice![
            CompiledFunction::new(0, 2, 1, 2, bslice![])
        ],
        ffi_functions: bslice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: bslice![],
    };
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: VMThread<DefaultAlloc> = create_vm_main_thread(alloc, &program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[Value::new_int(114), Value::new_int(514)]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as u64),
                       ValueTypeTag::Int as u64);
            assert_eq!(result[0].vt_data.inner.int_value, 114 + 514);
        }
    } else {
        panic!()
    }
}

async fn basic_fn_call() {
    let program: CompiledProgram<DefaultAlloc> = CompiledProgram {
        code: bslice![
                                                       // application_start() -> (int)
            /*00*/ Insc::MakeIntConst(1, 0),           // %0 = $1
            /*01*/ Insc::MakeIntConst(2, 1),           // %1 = $2
            /*02*/ Insc::Call(1, bslice![0, 1], bslice![0]), // [ %0 ] = call sum(%0, %1)
            /*03*/ Insc::Return(bslice![0]),              // return [ %0 ]

                                                       // sum(%0, %1) -> (int)
            /*04*/ Insc::AddInt(0, 1, 0),              // [ %0 ] = add int %0, %1
            /*05*/ Insc::Return(bslice![0])               // return [ %0 ]
        ],
        const_pool: bslice![],
        init_proc: 0,
        functions: bslice![
            CompiledFunction::new(0, 0, 1, 2, bslice![]), // application_start
            CompiledFunction::new(4, 2, 1, 2, bslice![]), // sum
        ],
        ffi_functions: bslice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: bslice![],
    };

    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: VMThread<DefaultAlloc> = create_vm_main_thread(alloc, &program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as u64),
                       ValueTypeTag::Int as u64);
            assert_eq!(result[0].vt_data.inner.int_value, 3);
        }
    } else {
        panic!()
    }
}

async fn fibonacci_call() {
    let fib_program: CompiledProgram<DefaultAlloc> = fibonacci_program();
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: VMThread<DefaultAlloc> = create_vm_main_thread(alloc, &fib_program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[Value::new_int(7)]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as u64),
                       ValueTypeTag::Int as u64);
            assert_eq!(result[0].vt_data.inner.int_value, 13);
        }
    } else {
        panic!()
    }
}

#[test] fn test_basic_program_eval() {
    block_on_future(basic_program_eval());
}

#[test] fn test_basic_fn_call() {
    block_on_future(basic_fn_call());
}

#[test] fn test_fibonacci_call() {
    block_on_future(fibonacci_call());
}
