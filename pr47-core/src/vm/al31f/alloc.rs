pub mod default_alloc;
pub mod no_gc_alloc;

use xjbutil::fat_ptr::FatPointer;

use crate::vm::al31f::stack::Stack;

/// Abstract memory manager of `AL31F` engine
pub trait Alloc: 'static + Send + Sync {
    /// Add one stack to `Alloc` management
    unsafe fn add_stack(&mut self, stack: *const Stack);
    /// Remove one stack from `Alloc` management
    unsafe fn remove_stack(&mut self, stack: *const Stack);
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
