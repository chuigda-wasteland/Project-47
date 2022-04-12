#![allow(improper_ctypes)]

use std::task::Context;
use crate::data::Value;

macro_rules! make_opaque_type {
    ($name:ident) => {
        #[repr(transparent)]
        pub struct $name { _private: [u8; 0] }
    }
}

make_opaque_type!(Stack);
make_opaque_type!(Allocator);

extern "C" {
    pub fn pr47_al31fu_cxx_allocator_new() -> *mut Allocator;

    pub fn pr47_al31fu_cxx_allocator_delete(allocator: *mut Allocator);

    pub fn pr47_al31fu_cxx_allocator_add_stack(allocator: *mut Allocator, stack: *mut Stack);

    pub fn pr47_al31fu_cxx_allocator_mark_object(allocator: *mut Allocator, object: Value);

    pub fn pr47_al31fu_cxx_poll_unsafe(
        coroutine_context: *mut (),
        allocator: *mut Stack,
        program: *mut (),
        stack: *mut Stack,

        poll_cx: *mut Context<'_>
    ) -> bool;
}
