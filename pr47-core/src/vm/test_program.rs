use crate::boxed_slice as bslice;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram};
use crate::vm::al31f::insc::Insc;

pub fn fibonacci_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: bslice![
                                                          // fibonacci(%0) -> (int)
            /*00*/ Insc::MakeIntConst(0, 1),              // %1 = $0
            /*01*/ Insc::EqValue(0, 1, 2),                // %2 = eq int %0, %1
            /*02*/ Insc::JumpIfTrue(2, 12),               // if %2 goto 12
            /*03*/ Insc::MakeIntConst(1, 1),              // %1 = $1
            /*04*/ Insc::EqValue(0, 1, 2),                // %2 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(2, 12),               // if %2 goto 12
            /*06*/ Insc::SubInt(0, 1, 2),                 // %2 = sub int %0, %1
            /*07*/ Insc::MakeIntConst(2, 1),              // %1 = $2
            /*08*/ Insc::SubInt(0, 1, 3),                 // %3 = sub int %0, %1
            /*09*/ Insc::Call(0, bslice![2], bslice![2]), // [ %2 ] = call fibonacci(%2)
            /*10*/ Insc::Call(0, bslice![3], bslice![3]), // [ %3 ] = call fibonacci(%3)
            /*11*/ Insc::AddInt(2, 3, 1),                 // %1 = add %2, %3
            /*12*/ Insc::ReturnOne(1)                     // return %1
        ],
        const_pool: bslice![],
        init_proc: 0,
        functions: bslice![
            CompiledFunction::new(0, 1, 1, 4, bslice![])
        ],
        ffi_functions: bslice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: bslice![],
    }
}
