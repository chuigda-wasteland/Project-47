use std::any::TypeId;

use crate::data::Value;
use crate::data::exception::Exception;
use crate::data::wrapper::DynBase;
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};
use crate::ds::object::Object;
use crate::util::async_utils::block_on_future;
use crate::vm::al31f::alloc::default_alloc::DefaultAlloc;
use crate::vm::al31f::compiled::CompiledProgram;
use crate::vm::al31f::executor::{VMThread, create_vm_main_thread, vm_thread_run_function};
use crate::vm::al31f::test_program::{
    basic_fn_call_program,
    basic_program,
    exception_no_eh_program,
    exception_program,
    fibonacci_program
};

async fn basic_program_eval() {
    let program: CompiledProgram<DefaultAlloc> = basic_program::<>();
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: Box<VMThread<DefaultAlloc>> = create_vm_main_thread(alloc, &program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[Value::new_int(114), Value::new_int(514)]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as usize),
                       ValueTypeTag::Int as usize);
            assert_eq!(result[0].vt_data.inner.int_value, 114 + 514);
        }
    } else {
        panic!()
    }
}

async fn basic_fn_call() {
    let program: CompiledProgram<DefaultAlloc> = basic_fn_call_program::<>();

    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: Box<VMThread<DefaultAlloc>> = create_vm_main_thread(alloc, &program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as usize),
                       ValueTypeTag::Int as usize);
            assert_eq!(result[0].vt_data.inner.int_value, 3);
        }
    } else {
        panic!()
    }
}

async fn fibonacci_call() {
    let fib_program: CompiledProgram<DefaultAlloc> = fibonacci_program();
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: Box<VMThread<DefaultAlloc>> =
        create_vm_main_thread(alloc, &fib_program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[Value::new_int(7)]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as usize),
                       ValueTypeTag::Int as usize);
            assert_eq!(result[0].vt_data.inner.int_value, 13);
        }
    } else {
        panic!()
    }
}

async fn exception_no_eh_call() {
    let exception_no_eh_program: CompiledProgram<DefaultAlloc> = exception_no_eh_program();
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: Box<VMThread<DefaultAlloc>> =
        create_vm_main_thread(alloc, &exception_no_eh_program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[]).await
    };

    if let Err(Exception::CheckedException(e /*: Value*/)) = result {
        unsafe {
            let dyn_base: *mut dyn DynBase = e.get_as_dyn_base();
            assert_eq!(
                dyn_base.as_ref().unwrap().dyn_type_id(),
                TypeId::of::<Object>()
            );
        };
    } else {
        panic!()
    }
}

async fn exception_call() {
    let exception_program: CompiledProgram<DefaultAlloc> = exception_program();
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: Box<VMThread<DefaultAlloc>> =
        create_vm_main_thread(alloc, &exception_program).await;
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &[]).await
    };
    if let Ok(result /*: Vec<Value>*/) = result {
        assert_eq!(result.len(), 1);
        assert!(result[0].is_value());
        unsafe {
            assert_eq!(result[0].vt_data.tag & (VALUE_TYPE_TAG_MASK as usize),
                       ValueTypeTag::Int as usize);
            assert_eq!(result[0].vt_data.inner.int_value, 114514);
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

#[test] fn test_exception_no_eh() {
    block_on_future(exception_no_eh_call());
}

#[test] fn test_exception() {
    block_on_future(exception_call());
}