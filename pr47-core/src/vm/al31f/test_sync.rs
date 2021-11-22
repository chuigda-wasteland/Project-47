use crate::data::exception::Exception;
use crate::data::Value;
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};
use crate::vm::al31f::alloc::default_alloc::DefaultAlloc;
use crate::vm::al31f::compiled::CompiledProgram;
use crate::vm::al31f::executor::vm_run_function_sync;
use crate::vm::al31f::test_program::basic_program;

#[test]
fn test_basic_program_eval() {
    let program: CompiledProgram<DefaultAlloc> = basic_program::<>();
    let alloc: DefaultAlloc = DefaultAlloc::new();

    let result: Result<Vec<Value>, Exception> = unsafe {
        vm_run_function_sync(alloc, &program, 0, &[Value::new_int(114), Value::new_int(514)])
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
