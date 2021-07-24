use std::ptr::NonNull;

use crate::data::Value;
use crate::data::tyck::TyckInfo;
// use crate::ffi::sync_fn::Function as FFIFunction;
use crate::vm::al31f::insc::Insc;

// #[cfg(feature = "async")]
// use crate::ffi::async_fn::AsyncFunction as FFIAsyncFunction;

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

pub struct CompiledProgram {
    pub code: Box<[Insc]>,
    pub const_pool: Box<[Value]>,
    pub init_proc: usize,
    pub functions: Box<[CompiledFunction]>,

    // TODO define a VM context first
    // ffi_functions: Box<[Box<dyn FFIFunction>]>,
    // #[cfg(feature = "async")]
    // async_ffi_funcs: Box<[Box<dyn FFIAsyncFunction>]>
}
