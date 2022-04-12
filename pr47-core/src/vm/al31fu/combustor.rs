use crate::data::Value;
use crate::ffi::async_fn::{AsyncVMContext, LockedCtx};
use crate::ffi::sync_fn::VMContext;
use crate::util::serializer::{CoroutineSharedData, Serializer};
use crate::vm::al31fu::exports::AsyncCombustor;
use crate::vm::al31fu::imports::{Alloc, Combustor};
use crate::vm::al31fu::imports::{pr47_al31fu_cxx_add_managed, pr47_al31fu_cxx_alloc_mark_object};

impl VMContext for Combustor {
    #[inline(always)]
    fn add_heap_managed(&mut self, value: Value) {
        unsafe {
            pr47_al31fu_cxx_add_managed(self.alloc, value);
        }
    }

    #[inline(always)]
    fn mark(&mut self, value: Value) {
        unsafe {
            pr47_al31fu_cxx_alloc_mark_object(self.alloc, value);
        }
    }
}

pub struct LockedAsyncContext {
    alloc: *mut Alloc
}

impl VMContext for LockedAsyncContext {
    #[inline(always)]
    fn add_heap_managed(&mut self, wide_ptr: Value) {
        unsafe {
            pr47_al31fu_cxx_add_managed(self.alloc, wide_ptr);
        }
    }

    #[inline(always)]
    fn mark(&mut self, wide_ptr: Value) {
        unsafe {
            pr47_al31fu_cxx_alloc_mark_object(self.alloc, wide_ptr);
        }
    }
}

unsafe impl Send for LockedAsyncContext {}
unsafe impl Sync for LockedAsyncContext {}

impl LockedCtx for LockedAsyncContext {}

impl AsyncVMContext for AsyncCombustor {
    type Locked = LockedAsyncContext;

    fn serializer(&self) -> &Serializer<(CoroutineSharedData, Self::Locked)> {
        &self.0
    }
}
