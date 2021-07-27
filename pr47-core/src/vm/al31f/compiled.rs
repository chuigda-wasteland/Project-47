use std::ptr::NonNull;

use crate::data::Value;
use crate::data::tyck::TyckInfo;
use crate::ffi::sync_fn::Function as FFIFunction;
use crate::vm::al31f::Combustor;
use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::insc::Insc;

#[cfg(feature = "async")] use crate::ffi::async_fn::AsyncFunction as FFIAsyncFunction;
#[cfg(feature = "async")] use crate::vm::al31f::AsyncCombustor;

pub struct CompiledFunction {
    pub start_addr: usize,
    pub arg_count: usize,
    pub ret_count: usize,
    pub stack_size: usize,

    pub param_tyck_info: Box<[Option<NonNull<TyckInfo>>]>
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
            param_tyck_info
        }
    }
}

pub struct CompiledProgram<A: Alloc> {
    pub code: Box<[Insc]>,
    pub const_pool: Box<[Value]>,
    pub init_proc: usize,
    pub functions: Box<[CompiledFunction]>,

    pub ffi_functions: Box<[Box<dyn FFIFunction<Combustor<A>>>]>,
    #[cfg(feature = "async")]
    pub async_ffi_funcs: Box<[Box<dyn FFIAsyncFunction<AsyncCombustor<A>>>]>
}
