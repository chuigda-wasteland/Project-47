use crate::boxed_slice;
use crate::data::Value;
use crate::data::exception::Exception;
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};
use crate::util::async_utils::block_on_future;
use crate::vm::al31f::VMThread;
use crate::vm::al31f::alloc::default_alloc::DefaultAlloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::executor::{create_vm_main_thread, vm_thread_run_function};
use crate::vm::al31f::insc::Insc;
use crate::vm::al31f::alloc::Alloc;

async fn basic_program_eval() {
    let program: CompiledProgram<DefaultAlloc> = CompiledProgram {
        code: boxed_slice![
            Insc::AddInt(0, 1, 0),
            Insc::Return(vec![0])
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 2, 1, 2, boxed_slice![])
        ],
        ffi_functions: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
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
        code: boxed_slice![
                                                       // application_start() -> (int)
            /*00*/ Insc::MakeIntConst(1, 0),           // %0 = $1
            /*01*/ Insc::MakeIntConst(2, 1),           // %1 = $2
            /*02*/ Insc::Call(1, vec![0, 1], vec![0]), // [ %0 ] = call sum(%0, %1)
            /*03*/ Insc::Return(vec![0]),              // return [ %0 ]

                                                       // sum(%0, %1) -> (int)
            /*04*/ Insc::AddInt(0, 1, 0),              // [ %0 ] = add int %0, %1
            /*05*/ Insc::Return(vec![0])               // return [ %0 ]
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 2, boxed_slice![]), // application_start
            CompiledFunction::new(4, 2, 1, 2, boxed_slice![]), // sum
        ],
        ffi_functions: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
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

fn fibonacci_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                    // fibonacci(%0) -> (int)
            /*00*/ Insc::MakeIntConst(0, 1),        // %1 = $0
            /*01*/ Insc::EqValue(0, 1, 2),          // %2 = eq int %0, %1
            /*02*/ Insc::JumpIfTrue(2, 12),         // if %2 goto 12
            /*03*/ Insc::MakeIntConst(1, 1),        // %1 = $1
            /*04*/ Insc::EqValue(0, 1, 2),          // %2 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(2, 12),         // if %2 goto 12
            /*06*/ Insc::SubInt(0, 1, 2),           // %2 = sub int %0, %1
            /*07*/ Insc::MakeIntConst(2, 1),        // %1 = $2
            /*08*/ Insc::SubInt(0, 1, 3),           // %3 = sub int %0, %1
            /*09*/ Insc::Call(0, vec![2], vec![2]), // [ %2 ] = call fibonacci(%2)
            /*10*/ Insc::Call(0, vec![3], vec![3]), // [ %3 ] = call fibonacci(%3)
            /*11*/ Insc::AddInt(2, 3, 1),           // %1 = add %2, %3
            /*12*/ Insc::Return(vec![1])            // return [ %1 ]
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 1, 1, 4, boxed_slice![])
        ],
        ffi_functions: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
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
