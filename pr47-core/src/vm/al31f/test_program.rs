use xjbutil::boxed_slice;
use xjbutil::slice_arena::SliceArena;
use xjbutil::void::Void;

use crate::builtins::object::Object;
use crate::data::exception::ExceptionInner;
use crate::data::Value;
use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfoPool;
use crate::ffi::{FFIException, Signature};
use crate::ffi::sync_fn::{FunctionBase, OwnershipGuard, VMContext, value_into_ref};
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::compiled::{CompiledFunction, CompiledProgram, ExceptionHandlingBlock};
use crate::vm::al31f::insc::Insc;

#[cfg(feature = "async")] use crate::ffi::async_fn::{
    AsyncFunctionBase,
    AsyncReturnType,
    AsyncVMContext,
    Promise
};
#[cfg(feature = "async")] use crate::ffi::async_fn::VMDataTrait;
use crate::std47::futures::SLEEP_MS_BIND;
use crate::std47::io::PRINT_BIND;

pub fn basic_program<A: Alloc>() -> CompiledProgram<A> {
    let (slice_arena, code) = unsafe {
        let slice_arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
            Insc::AddInt(0, 1, 0),
            Insc::Return(slice_arena.unsafe_make(&[0]))
        ];
        (slice_arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                             // application_start() -> (int)
            /*00*/ Insc::MakeIntConst(1, 0),                 // %0 = $1
            /*01*/ Insc::MakeIntConst(2, 1),                 // %1 = $2
            /*02*/ Insc::Call(1, arena.unsafe_make(&[0, 1]), // [ %0 ] = call sum(%0, %1)
                                 arena.unsafe_make(&[0])),
            /*03*/ Insc::Return(arena.unsafe_make(&[0])),    // return [ %0 ]

                                                             // sum(%0, %1) -> (int)
            /*04*/ Insc::AddInt(0, 1, 0),                    // [ %0 ] = add int %0, %1
            /*05*/ Insc::Return(arena.unsafe_make(&[0]))     // return [ %0 ]
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                           // fibonacci(%0) -> (int)
            /*00*/ Insc::MakeIntConst(0, 1),               // %1 = $0
            /*01*/ Insc::LeInt(0, 1, 2),                   // %2 = le int %0, %1
            /*02*/ Insc::JumpIfTrue(2, 12),                // if %2 goto L.12
            /*03*/ Insc::MakeIntConst(1, 1),               // %1 = $1
            /*04*/ Insc::EqValue(0, 1, 2),                 // %2 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(2, 12),                // if %2 goto L.12
            /*06*/ Insc::SubInt(0, 1, 2),                  // %2 = sub int %0, %1
            /*07*/ Insc::MakeIntConst(2, 1),               // %1 = $2
            /*08*/ Insc::SubInt(0, 1, 3),                  // %3 = sub int %0, %1
            /*09*/ Insc::Call(0, arena.unsafe_make(&[2]),
                                 arena.unsafe_make(&[2])), // [ %2 ] = call fibonacci(%2)
            /*10*/ Insc::Call(0, arena.unsafe_make(&[3]),
                                 arena.unsafe_make(&[3])), // [ %3 ] = call fibonacci(%3)
            /*11*/ Insc::AddInt(2, 3, 1),                  // %1 = add %2, %3
            /*12*/ Insc::ReturnOne(1)                      // return %1
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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
    let (slice_arena, code) = {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
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
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                          // foo() -> ()
            /*00*/ Insc::MakeIntConst(12345, 0),          // %0 = $12345
            /*01*/ Insc::Call(1, arena.unsafe_make(&[]),
                              arena.unsafe_make(&[])),    // call bar()
            /*02*/ Insc::ReturnOne(0),                    // return %0

                                                          // foo:eh:Object
            /*03*/ Insc::MakeIntConst(114514, 0),         // %0 = $114514
            /*04*/ Insc::ReturnOne(0),                    // return %0^


                                                          // bar() -> ()
            /*05*/ Insc::Call(2, arena.unsafe_make(&[]),
                                 arena.unsafe_make(&[])), // call baz()
            /*06*/ Insc::ReturnNothing,                   // return

                                                          // baz() -> !
            /*07*/ Insc::CreateObject(0),                 // %0 = create-object
            /*08*/ Insc::Raise(0)                         // raise %0
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                           // foo() -> (int)
            /*00*/ Insc::Call(1, arena.unsafe_make(&[]),   // %0 = call bar
                                 arena.unsafe_make(&[0])),
            /*01*/ Insc::ReturnOne(0),                     // return %0

                                                           // bar() -> !
            /*02*/ Insc::CreateObject(0),                  // %0 = create-object
            /*03*/ Insc::Raise(0),                         // raise %0
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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

const PR47BINDER_FFI_FUNCTION: &'static Pr47Binder_ffi_function = &Pr47Binder_ffi_function();

pub fn ffi_call_program<A: Alloc>() -> CompiledProgram<A> {
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                                    // main() -> ()
            /*00*/ Insc::CreateObject(0),                           // %0 = create-object
            /*01*/ Insc::FFICallRtlc(0,                             // ffi-call-rtlc @0(%0, %0, %0)
                                     arena.unsafe_make(&[0, 0, 0]),
                                     arena.unsafe_make(&[])),
            /*02*/ Insc::ReturnNothing                              // return
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![PR47BINDER_FFI_FUNCTION as _],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

pub fn bench_raw_iter_program<A: Alloc>() -> CompiledProgram<A> {
    let (slice_arena, code) = {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
            /*00*/ Insc::MakeIntConst(0, 0),               // %0 = $0
            /*01*/ Insc::MakeIntConst(100_000_000, 1),     // %1 = $100_000_000
            /*02*/ Insc::MakeIntConst(1, 2),               // %2 = $1
            /*03*/ Insc::EqValue(0, 1, 3),                 // %3 = eq int %0, %1
            /*04*/ Insc::JumpIfTrue(3, 7),                 // if %3 goto L.7
            /*05*/ Insc::AddInt(0, 2, 0),                  // %0 = add int %0, %2
            /*06*/ Insc::Jump(3),                          // goto L.3
            /*07*/ Insc::ReturnNothing                     // return
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
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
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
            /*00*/ Insc::MakeIntConst(0, 0),                        // %0 = $0
            /*01*/ Insc::MakeIntConst(100_000_000, 1),              // %1 = $100_000_000
            /*02*/ Insc::MakeIntConst(1, 2),                        // %2 = $1
            /*03*/ Insc::CreateObject(3),                           // %3 = create-object
            /*04*/ Insc::EqValue(0, 1, 4),                          // %4 = eq int %0, %1
            /*05*/ Insc::JumpIfTrue(4, 9),                          // if %4 goto L.9
            /*06*/ Insc::FFICallRtlc(0,                             // ffi-call-rtlc @0(%3, %3, %3)
                                     arena.unsafe_make(&[3, 3, 3]),
                                     arena.unsafe_make(&[])),
            /*07*/ Insc::AddInt(0, 2, 0),                           // %0 = add int %0, %2
            /*08*/ Insc::Jump(4),                                   // goto L.4
            /*09*/ Insc::ReturnNothing                              // return
        ];
        (arena, code)
    };


    CompiledProgram {
        slice_arena,
        code,
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 5, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![PR47BINDER_FFI_FUNCTION as _],
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

const PR47_BINDER_FFI_FUNCTION2: &'static Pr47Binder_ffi_function2 = &Pr47Binder_ffi_function2();

pub fn ffi_call_program2<A: Alloc>() -> CompiledProgram<A> {
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                                 // application_start(%0, %1) -> i64
            /*00*/ Insc::FFICallRtlc(0,                          // %0 = ffi-call-rtlc @0(%0, %1)
                                     arena.unsafe_make(&[0, 1]),
                                     arena.unsafe_make(&[0])),
            /*01*/ Insc::ReturnOne(0)                            // return %0
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 2, 1, 2, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![PR47_BINDER_FFI_FUNCTION2 as _],
        #[cfg(feature="async")] async_ffi_funcs: boxed_slice![]
    }
}

pub fn bench_ffi_call_program2<A: Alloc>() -> CompiledProgram<A> {
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
            /*00*/ Insc::MakeIntConst(0, 0),                        // %0 = $0
            /*01*/ Insc::MakeIntConst(10_000, 1),                   // %1 = $1
            /*02*/ Insc::EqValue(0, 1, 2),                          // %2 = eq int %0, %1
            /*03*/ Insc::JumpIfTrue(2, 15),                         // if %2 goto L.15
            /*04*/ Insc::MakeIntConst(0, 3),                        // %3 = $0
            /*05*/ Insc::EqValue(3, 1, 2),                          // %2 = eq int %3, %1
            /*06*/ Insc::JumpIfTrue(2, 13),                         // if %2 goto L.13
            /*07*/ Insc::AddInt(0, 3, 4),                           // %4 = add int %0, %3
            /*08*/ Insc::FFICallRtlc(0, arena.unsafe_make(&[0, 3]), // %5 = ffi-call-rtlc @0(%0, %3)
                                     arena.unsafe_make(&[5])),
            /*09*/ Insc::EqValue(4, 5, 2),                          // %2 = eq int %4, %5
            /*10*/ Insc::JumpIfFalse(2, 16),                        // if !%2 goto L.16
            /*11*/ Insc::IncrInt(3),                                // inc int %3
            /*12*/ Insc::Jump(5),                                   // goto L.5
            /*13*/ Insc::IncrInt(0),                                // inc int %0
            /*14*/ Insc::Jump(2),                                   // goto L.2
            /*15*/ Insc::ReturnNothing,                             // ret
            /*16*/ Insc::CreateObject(0),                           // %0 = create-object
            /*17*/ Insc::Raise(0)                                   // raise %0
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 6, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![PR47_BINDER_FFI_FUNCTION2 as _],
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
        struct AsyncRet {
            r: Result<String, std::io::Error>
        }

        impl<A: Alloc> AsyncReturnType<A> for AsyncRet {
            fn is_err(&self) -> bool {
                self.r.is_err()
            }

            fn resolve(self, alloc: &mut A, dests: &[*mut Value]) -> Result<usize, ExceptionInner> {
                match self.r {
                    Ok(data) => {
                        let value: Value = Value::new_owned(data);
                        unsafe {
                            alloc.add_managed(value.ptr_repr);
                            **dests.get_unchecked(0) = value;
                        }
                        Ok(1)
                    },
                    Err(e) => {
                        let err_value: Value = Value::new_owned(e);
                        unsafe {
                            alloc.add_managed(err_value.ptr_repr);
                        }
                        Err(ExceptionInner::Checked(err_value))
                    }
                }
            }
        }

        let fut = async move {
            let r: Result<String, std::io::Error> = async_ffi_function().await;
            Box::new(AsyncRet { r }) as Box<dyn AsyncReturnType<A>>
        };

        Ok(Promise(Box::pin(fut)))
    }
}

const PR47BINDER_ASYNC_FFI_FUNCTION: &'static Pr47Binder_async_ffi_function
    = &Pr47Binder_async_ffi_function();

#[cfg(feature = "async")]
pub fn async_ffi_call_program<A: Alloc>() -> CompiledProgram<A> {
    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                                  // application_start() -> string
            /*00*/ Insc::FFICallAsync(0,                          // %0 = ffi-call-async @0()),
                                      arena.unsafe_make(&[]), 0),
            /*01*/ Insc::Await(0, arena.unsafe_make(&[0])),       // %0 = await %0
            /*02*/ Insc::ReturnOne(0)                             // ret string %0
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
        const_pool: boxed_slice![],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 1, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![],
        async_ffi_funcs: boxed_slice![PR47BINDER_ASYNC_FFI_FUNCTION as _]
    }
}

#[cfg(all(feature = "async", feature = "al31f-builtin-ops"))]
pub fn async_spawn_program<A: Alloc>() -> CompiledProgram<A> {
    let string1: String = "string1\n".into();
    let string1: Value = Value::new_owned(string1);

    let string2: String = "string2\n".into();
    let string2: Value = Value::new_owned(string2);

    let string3: String = "string3\n".into();
    let string3: Value = Value::new_owned(string3);

    let string4: String = "string4\n".into();
    let string4: Value = Value::new_owned(string4);

    let (slice_arena, code) = unsafe {
        let arena: SliceArena<8192, 8> = SliceArena::new();
        let code: Box<[Insc]> = boxed_slice![
                                                               // application_start()
            /*00*/ Insc::MakeIntConst(1, 0),                   // %0 = $1
            /*01*/ Insc::Spawn(1, arena.unsafe_make(&[])),     // spawn $1, []
            /*02*/ Insc::Await(0, arena.unsafe_make(&[0])),    // %0 = #spawn-result
            /*03*/ Insc::LoadConst(0, 1),                      // %1 = load-const .string1
            /*04*/ Insc::FFICallRtlc(0,                        // ffi-call print(%1)
                                     arena.unsafe_make(&[1]),
                                     arena.unsafe_make(&[])),
            /*05*/ Insc::MakeIntConst(1000, 1),                // %1 = $1000
            /*06*/ Insc::FFICallAsync(0,                       // %1 = ffi-call-async sleep_ms(%1)
                                      arena.unsafe_make(&[1]),
                                      1),
            /*07*/ Insc::Await(1, arena.unsafe_make(&[])),     // await %1
            /*08*/ Insc::LoadConst(1, 1),                      // %1 = load-const .string2
            /*09*/ Insc::FFICallRtlc(0,                        // ffi-call print(%1)
                                     arena.unsafe_make(&[1]),
                                     arena.unsafe_make(&[])),
            /*10*/ Insc::Await(0, arena.unsafe_make(&[])),     // await %0
            /*11*/ Insc::ReturnNothing,                        // ret

                                                               // spawned_task_main()
            /*12*/ Insc::LoadConst(2, 0),                      // %0 = load-const .string3
            /*13*/ Insc::FFICallRtlc(0,                        // ffi-call print(%0)
                                     arena.unsafe_make(&[0]),
                                     arena.unsafe_make(&[])),
            /*14*/ Insc::MakeIntConst(1000, 0),                // %0 = $1000
            /*15*/ Insc::FFICallAsync(0,                       // %0 = ffi-call-async sleep_ms(%0)
                                      arena.unsafe_make(&[0]),
                                      0),
            /*16*/ Insc::Await(0, arena.unsafe_make(&[])),     // await %0
            /*17*/ Insc::LoadConst(3, 0),                      // %0 = load-const .string4
            /*18*/ Insc::FFICallRtlc(0,                        // ffi-call print(%0)
                                     arena.unsafe_make(&[0]),
                                     arena.unsafe_make(&[])),
            /*19*/ Insc::ReturnNothing                         // ret
        ];
        (arena, code)
    };

    CompiledProgram {
        slice_arena,
        code,
        const_pool: boxed_slice![string1, string2, string3, string4],
        init_proc: 0,
        functions: boxed_slice![
            CompiledFunction::new(0, 0, 0, 2, boxed_slice![]),
            CompiledFunction::new(12, 0, 0, 1, boxed_slice![])
        ],
        ffi_funcs: boxed_slice![PRINT_BIND as _],
        async_ffi_funcs: boxed_slice![SLEEP_MS_BIND as _]
    }
}
