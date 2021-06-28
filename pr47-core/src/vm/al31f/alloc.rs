use std::collections::{HashSet, VecDeque};
use std::mem::transmute;

use crate::vm::al31f::stack::Stack;
use crate::util::mem::FatPointer;

/// Abstract memory manager of `AL31F` engine
pub trait Alloc {
    /// Add one stack to `Alloc` management
    unsafe fn add_stack(&mut self, stack: *const Stack<'_>);
    /// Remove one stack from `Alloc` management
    unsafe fn remove_stack(&mut self, stack: *const Stack<'_>);
    /// Make the object denoted by `data` pointer managed
    unsafe fn add_managed(&mut self, data: FatPointer);
    /// Mark the object denoted by `data` as useful when it gets added into some container. This
    /// method is used by tri-color GC.
    unsafe fn mark_object(&mut self, data: FatPointer);
    /// Perform garbage collection
    unsafe fn collect(&mut self);
    /// Allow or disallow garbage collection
    fn set_gc_allowed(&mut self, allowed: bool);
}

/// Default allocator for `AL31F`, with STW GC.
pub struct DefaultAlloc {
    stacks: HashSet<*const Stack<'static>>,
    managed: HashSet<FatPointer>,
    debt: usize,
    max_debt: usize,
    gc_allowed: bool
}

pub const DEFAULT_MAX_DEBT: usize = 512;

impl DefaultAlloc {
    pub fn new() -> Self {
        Self::with_max_debt(DEFAULT_MAX_DEBT)
    }

    pub fn with_max_debt(max_debt: usize) -> Self {
        Self {
            stacks: HashSet::new(),
            managed: HashSet::new(),
            debt: 0,
            max_debt,
            gc_allowed: false
        }
    }
}

impl Default for DefaultAlloc {
    fn default() -> Self {
        Self::new()
    }
}

// TODO
// impl Drop for DefaultAlloc {
//     ...
// }

impl Alloc for DefaultAlloc {
    unsafe fn add_stack(&mut self, stack: *const Stack<'_>) {
        self.stacks.insert(transmute::<>(stack));
    }

    unsafe fn remove_stack(&mut self, stack: *const Stack<'_>) {
        let removed = self.stacks.remove(&transmute::<>(stack));
        debug_assert!(removed);
    }

    unsafe fn add_managed(&mut self, data: FatPointer) {
        if self.max_debt < self.debt && self.gc_allowed {
            self.collect();
        }
        self.managed.insert(data);
    }

    unsafe fn mark_object(&mut self, _data: FatPointer) {
        // do nothing
    }

    unsafe fn collect(&mut self) {
        let mut _to_collect: VecDeque<FatPointer> = VecDeque::new();
    }

    fn set_gc_allowed(&mut self, allowed: bool) {
        self.gc_allowed = allowed;
    }
}
