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
make_opaque_type!(Alloc);

extern "C" {
    pub fn pr47_al31fu_cxx_alloc_new() -> *mut Alloc;

    pub fn pr47_al31fu_cxx_alloc_delete(alloc: *mut Alloc);

    pub fn pr47_al31fu_cxx_alloc_add_stack(alloc: *mut Alloc, stack: *mut Stack);

    pub fn pr47_al31fu_cxx_alloc_remove_stack(alloc: *mut Alloc, stack: *mut Stack);

    pub fn pr47_al31fu_cxx_alloc_mark_object(alloc: *mut Alloc, object: Value);

    pub fn pr47_al31fu_cxx_poll_unsafe(
        coroutine_context: *mut (),
        alloc: *mut Alloc,
        program: *mut (),
        stack: *mut Stack,

        poll_cx: *mut Context<'_>
    ) -> bool;
}
