use crate::boxed_slice;
use crate::data::Value;
use crate::data::exception::Exception;
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};
use crate::vm::al31f::VMThread;
use crate::vm::al31f::alloc::default_alloc::DefaultAlloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::executor::{create_vm_main_thread, vm_thread_run_function};
use crate::vm::al31f::insc::Insc;

async fn basic_program_eval() {
    let program: CompiledProgram<DefaultAlloc> = CompiledProgram {
        code: boxed_slice![
            Insc::AddInt(0, 1, 0),
            Insc::Return(vec![0])
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 2, 1, 2, boxed_slice!())
        ],
        ffi_functions: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    };
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let mut vm_thread: VMThread<DefaultAlloc> = create_vm_main_thread(alloc, &program).await;
    let params: [Value; 2] = [Value::new_int(114), Value::new_int(514)];
    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_thread_run_function(&mut vm_thread, 0, &params).await
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

#[test] fn test_basic_program_eval() {
    use crate::util::async_utils::block_on_future;
    block_on_future(basic_program_eval());
}
