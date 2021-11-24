use xjbutil::boxed_slice;
use xjbutil::void::Void;

use crate::builtins::object::Object;
use crate::data::Value;
use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfoPool;
use crate::ffi::{FFIException, Signature};
use crate::ffi::sync_fn::{Function, FunctionBase, OwnershipGuard, VMContext, value_into_ref};
use crate::vm::al31f::{AL31F, Combustor};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram, ExceptionHandlingBlock};
use crate::vm::al31f::insc::Insc;

#[cfg(feature = "async")] use crate::ffi::async_fn::{
    AsyncFunction,
    AsyncFunctionBase,
    AsyncReturnType,
    AsyncVMContext,
    Promise,
    PromiseGuard
};
#[cfg(feature = "async")] use crate::ffi::async_fn::{PromiseContext, VMDataTrait};
#[cfg(feature = "async")] use crate::vm::al31f::AsyncCombustor;

pub fn basic_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
            Insc::AddInt(0, 1, 0),
            Insc::Return(boxed_slice![0])
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 2, 1, 2, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn basic_fn_call_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                              // application_start() -> (int)
            /*00*/ Insc::MakeIntConst(1, 0),                  // %0 = $1
            /*01*/ Insc::MakeIntConst(2, 1),                  // %1 = $2
            /*02*/ Insc::Call(
                       1, boxed_slice![0, 1], boxed_slice![0] // [ %0 ] = call sum(%0, %1)
                   ),
            /*03*/ Insc::Return(boxed_slice![0]),             // return [ %0 ]

                                                              // sum(%0, %1) -> (int)
            /*04*/ Insc::AddInt(0, 1, 0),                     // [ %0 ] = add int %0, %1
            /*05*/ Insc::Return(boxed_slice![0])              // return [ %0 ]
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 2, boxed_slice![]), // application_start
            CompiledFunction::new(4, 2, 1, 2, boxed_slice![]), // sum
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn fibonacci_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                                    // fibonacci(%0) -> (int)
            /*00*/ Insc::MakeIntConst(0, 1),                        // %1 = $0
            /*01*/ Insc::LeInt(0, 1, 2),                            // %2 = le int %0, %1
            /*02*/ Insc::JumpIfTrue(2, 12),                         // if %2 goto L.12
            /*03*/ Insc::MakeIntConst(1, 1),                        // %1 = $1
            /*04*/ Insc::EqValue(0, 1, 2),                          // %2 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(2, 12),                         // if %2 goto L.12
            /*06*/ Insc::SubInt(0, 1, 2),                           // %2 = sub int %0, %1
            /*07*/ Insc::MakeIntConst(2, 1),                        // %1 = $2
            /*08*/ Insc::SubInt(0, 1, 3),                           // %3 = sub int %0, %1
            /*09*/ Insc::Call(0, boxed_slice![2], boxed_slice![2]), // [ %2 ] = call fibonacci(%2)
            /*10*/ Insc::Call(0, boxed_slice![3], boxed_slice![3]), // [ %3 ] = call fibonacci(%3)
            /*11*/ Insc::AddInt(2, 3, 1),                           // %1 = add %2, %3
            /*12*/ Insc::ReturnOne(1)                               // return %1
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 1, 1, 4, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn alloc_1m_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                      // alloc_1m()
            /*00*/ Insc::MakeIntConst(0, 0),          // %0 = $0
            /*01*/ Insc::MakeIntConst(1, 1),          // %1 = $1
            /*02*/ Insc::MakeIntConst(10_000_000, 2), // %2 = $10_000_000
            /*03*/ Insc::EqValue(0, 2, 3),            // %3 = eq value %0, %2
            /*04*/ Insc::JumpIfTrue(3, 8),            // if %3 goto L.8
            /*05*/ Insc::CreateObject(3),             // %3 = new object
            /*06*/ Insc::SubInt(2, 1, 2),             // %2 = sub int %2, %1
            /*07*/ Insc::Jump(3),                     // goto L.3
            /*08*/ Insc::ReturnNothing                // return
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 4, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![],
    }
}

pub fn exception_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                                  // foo() -> ()
            /*00*/ Insc::MakeIntConst(12345, 0),                  // %0 = $12345
            /*01*/ Insc::Call(1, boxed_slice![], boxed_slice![]), // call bar()
            /*02*/ Insc::ReturnOne(0),                            // return %0

                                                                  // foo:eh:Object
            /*03*/ Insc::MakeIntConst(114514, 0),                 // %0 = $114514
            /*04*/ Insc::ReturnOne(0),                            // return %0


                                                                  // bar() -> ()
            /*05*/ Insc::Call(2, boxed_slice![], boxed_slice![]), // call baz()
            /*06*/ Insc::ReturnNothing,                           // return

                                                                  // baz() -> !
            /*07*/ Insc::CreateObject(0),                         // %0 = create-object
            /*08*/ Insc::Raise(0)                                 // raise %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new_with_exc(0, 0, 1, 1, boxed_slice![], boxed_slice![
                ExceptionHandlingBlock::new(0, 2, <Void as StaticBase<Object>>::type_id(), 3)
            ]),
            CompiledFunction::new(5, 0, 0, 0, boxed_slice![]),
            CompiledFunction::new(7, 0, 0, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![]
    }
}

pub fn exception_no_eh_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                                   // foo() -> (int)
            /*00*/ Insc::Call(1, boxed_slice![], boxed_slice![0]), // %0 = call bar
            /*01*/ Insc::ReturnOne(0),

                                                                   // bar() -> !
            /*02*/ Insc::CreateObject(0),                          // %0 = create-object
            /*03*/ Insc::Raise(0),                                 // raise %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 1, boxed_slice![]),
            CompiledFunction::new(2, 0, 1, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature = "async")]
        async_ffi_funcs: boxed_slice![]
    }
}

#[inline(never)] fn ffi_function(x: &Object, y: &Object, z: &Object) {
    assert_eq!(x as *const Object as usize, y as *const Object as usize);
    assert_eq!(y as *const Object as usize, z as *const Object as usize);
}

#[allow(non_camel_case_types)]
struct Pr47Binder_ffi_function();

impl FunctionBase for Pr47Binder_ffi_function {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        todo!()
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        debug_assert_eq!(args.len(), 3);
        debug_assert_eq!(rets.len(), 0);

        let (a1, g1): (&Object, Option<OwnershipGuard>) = value_into_ref(*args.get_unchecked(0))?;
        let (a2, g2): (&Object, Option<OwnershipGuard>) = value_into_ref(*args.get_unchecked(1))?;
        let (a3, g3): (&Object, Option<OwnershipGuard>) = value_into_ref(*args.get_unchecked(2))?;

        ffi_function(a1, a2, a3);

        std::mem::drop(g3);
        std::mem::drop(g2);
        std::mem::drop(g1);

        Ok(())
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        _context: &mut CTX,
        _args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        todo!()
    }
}

pub fn ffi_call_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                               // main() -> ()
            /*00*/ Insc::CreateObject(0),                      // %0 = create-object
            /*01*/ Insc::FFICallRtlc(0, boxed_slice![0, 0, 0], // ffi-call-rtlc @0(%0, %0, %0)
                                     boxed_slice![]),
            /*02*/ Insc::ReturnNothing                         // return
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![
            Box::new(Pr47Binder_ffi_function()) as Box<dyn Function<Combustor<A>>>
        ],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

pub fn bench_raw_iter_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
            /*00*/ Insc::MakeIntConst(0, 0),               // %0 = $0
            /*01*/ Insc::MakeIntConst(100_000_000, 1),     // %1 = $100_000_000
            /*02*/ Insc::MakeIntConst(1, 2),               // %2 = $1
            /*03*/ Insc::EqValue(0, 1, 3),                 // %3 = eq int %0, %1
            /*04*/ Insc::JumpIfTrue(3, 7),                 // if %3 goto L.7
            /*05*/ Insc::AddInt(0, 2, 0),                  // %0 = add int %0, %2
            /*06*/ Insc::Jump(3),                          // goto L.3
            /*07*/ Insc::ReturnNothing                     // return
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 5, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

pub fn bench_ffi_call_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
            /*00*/ Insc::MakeIntConst(0, 0),                   // %0 = $0
            /*01*/ Insc::MakeIntConst(100_000_000, 1),         // %1 = $100_000_000
            /*02*/ Insc::MakeIntConst(1, 2),                   // %2 = $1
            /*03*/ Insc::CreateObject(3),                      // %3 = create-object
            /*04*/ Insc::EqValue(0, 1, 4),                     // %4 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(4, 9),                     // if %4 goto L.9
            /*06*/ Insc::FFICallRtlc(0, boxed_slice![3, 3, 3], // ffi-call-rtlc @0(%3, %3, %3)
                                     boxed_slice![]),
            /*07*/ Insc::AddInt(0, 2, 0),                      // %0 = add int %0, %2
            /*08*/ Insc::Jump(4),                              // goto L.4
            /*09*/ Insc::ReturnNothing                         // return
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 5, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![
            Box::new(Pr47Binder_ffi_function()) as Box<dyn Function<Combustor<A>>>
        ],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

#[inline(never)] fn ffi_function2(a: i64, b: i64) -> i64 {
    a + b
}

#[allow(non_camel_case_types)]
struct Pr47Binder_ffi_function2();

impl FunctionBase for Pr47Binder_ffi_function2 {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        unimplemented!()
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        debug_assert_eq!(args.len(), 2);
        debug_assert_eq!(rets.len(), 1);

        let a1: i64 = args.get_unchecked(0).vt_data.inner.int_value;
        let a2: i64 = args.get_unchecked(1).vt_data.inner.int_value;

        let ret: i64 = ffi_function2(a1, a2);
        *(*rets.get_unchecked(0)) = Value::new_int(ret);

        Ok(())
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        _context: &mut CTX,
        _args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        todo!()
    }
}

pub fn ffi_call_program2<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                            // application_start(%0, %1) -> i64
            /*00*/ Insc::FFICallRtlc(0, boxed_slice![0, 1], // %0 = ffi-call-rtlc @0(%0, %1)
                                     boxed_slice![0]),
            /*01*/ Insc::ReturnOne(0)                       // return %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 2, 1, 2, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![
            Box::new(Pr47Binder_ffi_function2()) as Box<dyn Function<Combustor<A>>>
        ],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

pub fn bench_ffi_call_program2<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
            /*00*/ Insc::MakeIntConst(0, 0),                // %0 = $0
            /*01*/ Insc::MakeIntConst(10_000, 1),           // %1 = $1
            /*02*/ Insc::EqValue(0, 1, 2),                  // %2 = eq int %0, %1
            /*03*/ Insc::JumpIfTrue(2, 15),                 // if %2 goto L.15
            /*04*/ Insc::MakeIntConst(0, 3),                // %3 = $0
            /*05*/ Insc::EqValue(3, 1, 2),                  // %2 = eq int %3, %1
            /*06*/ Insc::JumpIfTrue(2, 13),                 // if %2 goto L.13
            /*07*/ Insc::AddInt(0, 3, 4),                   // %4 = add int %0, %3
            /*08*/ Insc::FFICallRtlc(0, boxed_slice![0, 3], // %5 = ffi-call-rtlc @0(%0, %3)
                                     boxed_slice![5]),
            /*09*/ Insc::EqValue(4, 5, 2),                  // %2 = eq int %4, %5
            /*10*/ Insc::JumpIfFalse(2, 16),                // if !%2 goto L.16
            /*11*/ Insc::IncrInt(3),                        // inc int %3
            /*12*/ Insc::Jump(5),                           // goto L.5
            /*13*/ Insc::IncrInt(0),                        // inc int %0
            /*14*/ Insc::Jump(2),                           // goto L.2
            /*15*/ Insc::ReturnNothing,                     // ret
            /*16*/ Insc::CreateObject(0),                   // %0 = create-object
            /*17*/ Insc::Raise(0)                           // raise %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 6, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![
            Box::new(Pr47Binder_ffi_function2()) as Box<dyn Function<Combustor<A>>>
        ],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

#[cfg(feature = "async")]
#[inline(never)] async fn async_ffi_function() -> Result<String, std::io::Error> {
    tokio::fs::read_to_string("./Cargo.toml").await
}

#[cfg(feature = "async")]
#[allow(non_camel_case_types)]
struct Pr47Binder_async_ffi_function();

#[cfg(feature = "async")]
impl AsyncFunctionBase for Pr47Binder_async_ffi_function {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        unimplemented!()
    }

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc= A>, ACTX: AsyncVMContext<VMData = VD>> (
        _context: &ACTX,
        _args: &[Value]
    ) -> Result<Promise<A>, FFIException> {
        let fut = async move {
            let r: Result<String, std::io::Error> = async_ffi_function().await;
            match r {
                Ok(data) => AsyncReturnType(Ok(boxed_slice![
                    Value::new_owned(data)
                ])),
                Err(e) => AsyncReturnType(Err(FFIException::Left(
                    Value::new_owned(e)
                )))
            }
        };

        Ok(Promise {
            fut: Box::pin(fut),
            ctx: PromiseContext {
                guard: PromiseGuard {
                    guards: boxed_slice![],
                    reset_guard_count: 0
                },
                resolver: Some(|alloc: &mut A, values: &[Value]| {
                    alloc.add_managed(values.get_unchecked(0).ptr_repr)
                })
            }
        })
    }
}

#[cfg(feature = "async")]
pub fn async_ffi_call_program<A: Alloc>() -> CompiledProgram<A> {
    CompiledProgram {
        code: boxed_slice![
                                                             // application_start() -> string
            /*00*/ Insc::FFICallAsync(0, boxed_slice![], 0), // %0 = ffi-call-async @0()),
            /*01*/ Insc::Await(0, boxed_slice![0]),          // %0 = await %0
            /*02*/ Insc::ReturnOne(0)                        // ret string %0
        ],
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        async_ffi_funcs: boxed_slice![
            Box::new(Pr47Binder_async_ffi_function())
                as Box<dyn AsyncFunction<A, AL31F<A>, AsyncCombustor<A>>>
        ]
    }
}
