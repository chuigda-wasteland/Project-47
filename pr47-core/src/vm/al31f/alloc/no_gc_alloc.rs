use xjbutil::wide_ptr::WidePointer;

use crate::vm::al31f::alloc::Alloc;
use crate::vm::al31f::stack::Stack;

pub struct NoGCAlloc {
    managed: Vec<WidePointer>
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
        for _wide_ptr /*: &WidePointer*/ in self.managed.iter() {
            todo!()
        }
    }
}

impl Alloc for NoGCAlloc {
    #[inline(always)] unsafe fn add_stack(&mut self, _stack: *const Stack) {}

    #[inline(always)] unsafe fn remove_stack(&mut self, _stack: *const Stack) {}

    unsafe fn add_managed(&mut self, data: WidePointer) {
        self.managed.push(data);
    }

    #[inline(always)] unsafe fn mark_object(&mut self, _data: WidePointer) {}

    #[inline(always)] unsafe fn collect(&mut self) {}

    #[inline(always)] fn set_gc_allowed(&mut self, _allowed: bool) {}
}
