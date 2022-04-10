use std::any::TypeId;
use std::ptr::NonNull;
use xjbutil::slice_arena::SliceArena;

use crate::data::Value;
use crate::data::tyck::TyckInfo;
use crate::ffi::sync_fn::Function as FFIFunction;
use crate::vm::al31f::Combustor;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::insc::Insc;

#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncFunction as FFIAsyncFunction;
#[cfg(feature = "async")] use crate::vm::al31f::{AL31F, AsyncCombustor};

pub struct ExceptionHandlingBlock {
    pub insc_ptr_range: (usize, usize),
    pub exception_id: TypeId,
    pub handler_addr: usize
}

impl ExceptionHandlingBlock {
    pub fn new(
        insc_ptr_start: usize,
        insc_ptr_end: usize,
        exception_id: TypeId,
        handler_addr: usize
    ) -> Self {
        Self {
            insc_ptr_range: (insc_ptr_start, insc_ptr_end),
            exception_id,
            handler_addr
        }
    }
}

pub struct CompiledFunction {
    pub start_addr: usize,
    pub arg_count: usize,
    pub ret_count: usize,
    pub stack_size: usize,

    pub param_tyck_info: Box<[Option<NonNull<TyckInfo>>]>,
    pub exc_handlers: Option<Box<[ExceptionHandlingBlock]>>
}

impl CompiledFunction {
    pub fn new(
        start_addr: usize,
        arg_count: usize,
        ret_count: usize,
        stack_size: usize,
        param_tyck_info: Box<[Option<NonNull<TyckInfo>>]>
    ) -> Self {
        Self {
            start_addr,
            arg_count,
            ret_count,
            stack_size,
            param_tyck_info,
            exc_handlers: None
        }
    }

    pub fn new_with_exc(
        start_addr: usize,
        arg_count: usize,
        ret_count: usize,
        stack_size: usize,
        param_tyck_info: Box<[Option<NonNull<TyckInfo>>]>,
        exc_handlers: Box<[ExceptionHandlingBlock]>
    ) -> Self {
        Self {
            start_addr,
            arg_count,
            ret_count,
            stack_size,
            param_tyck_info,
            exc_handlers: Some(exc_handlers)
        }
    }
}

pub struct CompiledProgram<A: Alloc> {
    pub slice_arena: SliceArena<8192, 8>,

    pub code: Box<[Insc]>,
    pub const_pool: Box<[Value]>,
    pub init_proc: usize,
    pub functions: Box<[CompiledFunction]>,

    pub ffi_funcs: Box<[&'static dyn FFIFunction<Combustor<A>>]>,
    #[cfg(feature = "async")]
    pub async_ffi_funcs: Box<[&'static dyn FFIAsyncFunction<A, AL31F<A>, AsyncCombustor<A>>]>
}
