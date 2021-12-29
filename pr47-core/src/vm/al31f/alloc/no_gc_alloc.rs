use crate::data::Value;
use crate::vm::al31f::alloc::{Alloc, AllocPin};
use crate::vm::al31f::stack::Stack;

pub struct NoGCAlloc {
    managed: Vec<Value>
}

impl NoGCAlloc {
    pub fn new() -> Self {
        Self {
            managed: vec![]
        }
    }
}

impl Drop for NoGCAlloc {
    fn drop(&mut self) {
        for _value /*: &Value*/ in self.managed.iter() {
            todo!()
        }
    }
}

impl Alloc for NoGCAlloc {
    #[inline(always)] unsafe fn add_stack(&mut self, _stack: *const Stack) {}

    #[inline(always)] unsafe fn remove_stack(&mut self, _stack: *const Stack) {}

    unsafe fn add_managed(&mut self, data: Value) {
        self.managed.push(data);
    }

    #[inline(always)] unsafe fn mark_object(&mut self, _data: Value) {}

    #[inline(always)] unsafe fn pin_objects(&mut self, _pinned: AllocPin) {}

    #[inline(always)] unsafe fn collect(&mut self) {}

    #[inline(always)] fn set_gc_allowed(&mut self, _allowed: bool) {}
}

unsafe impl Send for NoGCAlloc {}
unsafe impl Sync for NoGCAlloc {}
