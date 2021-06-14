use std::collections::HashSet;

use crate::data::Value;
use crate::data::wrapper::DynBase;
use crate::vm::al31f::stack::Stack;
use crate::util::mem::FatPointer;

pub trait Alloc {
    unsafe fn add_stack(&mut self, stack: *const Stack<'_>);
    unsafe fn remove_stack(&mut self, stack: *const Stack<'_>);
    unsafe fn alloc(&mut self, data: *mut dyn DynBase) -> Value;
    unsafe fn alloc_raw(&mut self, data: FatPointer) -> Value;
    unsafe fn collect(&mut self);
}

pub struct DefaultAlloc {
    stacks: HashSet<*const Stack<'static>>,
    blks: HashSet<FatPointer>
}

impl DefaultAlloc {
    pub fn new() -> Self {
        Self {
            stacks: HashSet::new(),
            blks: HashSet::new()
        }
    }
}

// TODO
// impl Drop for DefaultAlloc {
//     ...
// }
// impl Alloc for DefaultAlloc {
//     ...
// }
