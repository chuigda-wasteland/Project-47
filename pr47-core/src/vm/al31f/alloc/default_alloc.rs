use std::collections::VecDeque;

use crate::data::Value;
use crate::data::generic::GenericTypeVT;
use crate::data::wrapper::{DynBase, OWN_INFO_COLLECT_MASK, OWN_INFO_GLOBAL_MASK};
use crate::vm::al31f::alloc::{Alloc, AllocPin};
use crate::vm::al31f::stack::Stack;

/// Default allocator for `AL31F`, with STW GC.
pub struct DefaultAlloc {
    stacks: Vec<*const Stack>,
    managed: Vec<Value>,
    pinned: Vec<AllocPin>,
    debt: usize,
    pin_debt: usize,
    max_debt: usize,
    max_pin_debt: usize,
    gc_allowed: bool
}

#[repr(u8)]
pub enum DefaultGCStatus {
    Unmarked = 0,
    Marked = 1
}

pub const DEFAULT_MAX_DEBT: usize = 1024;
pub const DEFAULT_MAX_PIN_DEBT: usize = 128;

impl DefaultAlloc {
    unsafe fn cleanup_pins(&mut self) {
        self.pinned.retain(|pinned: &AllocPin| *pinned.fixed());
        self.pin_debt = 0;
    }
}

impl DefaultAlloc {
    pub fn new() -> Self {
        Self::with_max_debt(DEFAULT_MAX_DEBT, DEFAULT_MAX_PIN_DEBT)
    }

    pub fn with_max_debt(max_debt: usize, max_pin_debt: usize) -> Self {
        Self {
            stacks: Vec::new(),
            managed: Vec::new(),
            pinned: Vec::new(),
            debt: 0,
            pin_debt: 0,
            max_debt,
            max_pin_debt,
            gc_allowed: false
        }
    }

    #[cfg(test)]
    pub fn contains_ptr(&self, ptr: xjbutil::wide_ptr::WidePointer) -> bool {
        self.managed.iter().map(|x| unsafe { x.ptr_repr }).any(|x| x == ptr)
    }
}

impl Default for DefaultAlloc {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DefaultAlloc {
    fn drop(&mut self) {
        // TODO should we extract this out? Or we use `Value` or so instead?
        for value /*: &Value*/ in self.managed.iter() {
            let ownership_info: u8 = unsafe { value.ownership_info() as u8 };

            if ownership_info & OWN_INFO_COLLECT_MASK == 0 {
                // TODO use `log` or `trace` here, don't panic. Memory leak is safe.
                panic!("failed to re-claim object {:?} on destruction, ownership_info = {:0b}",
                       unsafe { value.ptr_repr },
                       ownership_info);
            }

            if value.is_container() {
                unsafe {
                    let container: *mut () = value.untagged_ptr_field() as *mut _;
                    let vt: *const GenericTypeVT = value.ptr_repr.trivia as *const _;
                    ((*vt).drop_fn)(container);
                }
            } else {
                let boxed: Box<dyn DynBase> = unsafe {
                    let dyn_base: *mut dyn DynBase = value.get_as_dyn_base();
                    Box::from_raw(dyn_base)
                };
                drop(boxed);
            }
        }
    }
}

unsafe impl Send for DefaultAlloc {}
unsafe impl Sync for DefaultAlloc {}

impl Alloc for DefaultAlloc {
    unsafe fn add_stack(&mut self, stack: *const Stack) {
        self.stacks.push(stack);
        self.stacks.sort();
    }

    unsafe fn remove_stack(&mut self, stack: *const Stack) {
        let _removed = self.stacks.remove(self.stacks.binary_search(&stack).unwrap_unchecked());
    }

    #[inline(never)]
    unsafe fn add_managed(&mut self, data: Value) {
        if self.max_debt < self.debt && self.gc_allowed {
            self.collect();
        }
        self.managed.push(data);
        self.debt += 1;
    }

    #[inline(always)]
    unsafe fn mark_object(&mut self, _data: Value) {
        // do nothing
    }

    #[inline(never)]
    unsafe fn pin_objects(&mut self, pinned: &[Value]) -> *mut bool {
        self.pin_debt += 1;
        if self.pin_debt > self.max_pin_debt {
            self.cleanup_pins();
        }

        let pin: AllocPin = AllocPin::new(true, pinned);
        let ret_ptr: *mut bool = pin.as_ptr().ptr_fixed.as_ptr();
        self.pinned.push(pin);
        ret_ptr
    }

    #[inline(never)]
    unsafe fn collect(&mut self) {
        self.cleanup_pins();
        self.debt = 0;

        for value /*: &Value*/ in self.managed.iter() {
            value.set_gc_info(DefaultGCStatus::Unmarked as u8);
        }

        let mut to_scan: VecDeque<Value> = VecDeque::new();

        for stack /*: &*const Stack*/ in self.stacks.iter() {
            #[cfg(debug_assertions)]
            for stack_value /*: &Value*/ in (**stack).values.iter().flatten() {
                if !stack_value.is_null() && !stack_value.is_value() {
                    to_scan.push_back(*stack_value);
                }
            }

            #[cfg(not(debug_assertions))]
            for stack_value /*: &Value*/ in (**stack).values.iter() {
                if !stack_value.is_null() && !stack_value.is_value() {
                    to_scan.push_back(*stack_value);
                }
            }
        }

        for pin /*: &AllocPin*/ in self.pinned.iter() {
            for pinned_object /*: &Value*/ in pin.flex().iter() {
                if !pinned_object.is_null() && !pinned_object.is_value() {
                    to_scan.push_back(*pinned_object);
                }
            }
        }

        while !to_scan.is_empty() {
            let value: Value = to_scan.pop_front().unwrap();
            if value.is_null() || value.is_value() {
                continue;
            }

            let gc_info: u8 = value.gc_info() as u8;
            let ownership_info: u8 = value.ownership_info() as u8;

            if (gc_info == DefaultGCStatus::Marked as u8) ||
                (ownership_info & OWN_INFO_COLLECT_MASK == 0) ||
                (ownership_info & OWN_INFO_GLOBAL_MASK != 0)
            {
                continue;
            }

            value.set_gc_info(DefaultGCStatus::Marked as u8);
            if !value.is_container() {
                let dyn_base: *mut dyn DynBase = value.get_as_dyn_base();
                if let Some(children /*: Box<dyn Iterator>*/) = (*dyn_base).children() {
                    for child /*: Value*/ in children {
                        to_scan.push_back(child);
                    }
                }
            } else {
                let container_vt: *const GenericTypeVT = value.ptr_repr.trivia as *const _;
                let data: *const () = value.get_as_mut_ptr() as *const ();
                if let Some(children /*: Box<dyn Iterator> */) = ((*container_vt).children_fn)(data)
                {
                    for child /*: Value*/ in children {
                        to_scan.push_back(child);
                    }
                }
            }
        }

        self.managed.retain(|value: &Value| {
            let ownership_info: u8 = value.ownership_info() as u8;
            let gc_info: u8 = value.gc_info() as u8;
            if gc_info == DefaultGCStatus::Unmarked as u8 &&
                (ownership_info & OWN_INFO_COLLECT_MASK != 0) &&
                (ownership_info & OWN_INFO_GLOBAL_MASK == 0)
            {
                if value.is_container() {
                    let container: *mut () = value.untagged_ptr_field() as *mut _;
                    let vt: *const GenericTypeVT = value.ptr_repr.trivia as *const _;
                    ((*vt).drop_fn)(container);
                } else {
                    let dyn_base: *mut dyn DynBase = value.get_as_dyn_base();
                    let boxed: Box<dyn DynBase> = Box::from_raw(dyn_base);
                    drop(boxed);
                }
                false
            } else {
                true
            }
        });
    }

    #[inline(always)]
    fn set_gc_allowed(&mut self, allowed: bool) {
        self.gc_allowed = allowed;
    }
}

#[cfg(test)]
mod test {
    use xjbutil::mem::move_to_heap;
    use crate::builtins::test_container::{TestContainer, create_test_container_vt};
    use crate::data::Value;
    use crate::data::generic::GenericTypeVT;
    use crate::data::tyck::TyckInfoPool;
    use crate::data::wrapper::Wrapper;
    use crate::vm::al31f::alloc::Alloc;
    use crate::vm::al31f::alloc::default_alloc::DefaultAlloc;
    use crate::vm::al31f::stack::{Stack, StackSlice};

    #[test] fn test_default_collector_simple() {
        let mut alloc: DefaultAlloc = DefaultAlloc::new();
        let mut stack: Stack = Stack::new();

        let mut stack_slice: StackSlice = unsafe { stack.ext_func_call_grow_stack(0, 3, &[]) };

        let str1: Value = Value::new_owned::<String>("114".into());
        let str2: Value = Value::new_owned::<String>("514".into());
        let str3: Value = Value::new_owned::<String>("1919810".into());

        let mut container: TestContainer<String> = TestContainer::new();
        container.inner.elements.push(str1);
        container.inner.elements.push(str2);
        container.inner.elements.push(str3);

        let container: Value = Value::new_owned::<TestContainer<String>>(container);

        unsafe {
            alloc.add_stack(&stack);
            alloc.add_managed(str1);
            alloc.add_managed(str2);
            alloc.add_managed(str3);
            alloc.add_managed(container);

            stack_slice.set_value(0, container);
            alloc.collect();
            assert!(alloc.contains_ptr(str1.ptr_repr));
            assert!(alloc.contains_ptr(str2.ptr_repr));
            assert!(alloc.contains_ptr(str3.ptr_repr));
            assert!(alloc.contains_ptr(container.ptr_repr));

            stack_slice.set_value(0, str1);
            stack_slice.set_value(1, str2);
            stack_slice.set_value(2, str3);
            alloc.collect();
            assert!(alloc.contains_ptr(str1.ptr_repr));
            assert!(alloc.contains_ptr(str2.ptr_repr));
            assert!(alloc.contains_ptr(str3.ptr_repr));
            assert!(!alloc.contains_ptr(container.ptr_repr));

            stack_slice.set_value(1, Value::new_null());
            alloc.collect();
            assert!(alloc.contains_ptr(str1.ptr_repr));
            assert!(!alloc.contains_ptr(str2.ptr_repr));
            assert!(alloc.contains_ptr(str3.ptr_repr));
            assert!(!alloc.contains_ptr(container.ptr_repr));
        }
    }

    #[test] fn test_default_collector_custom_vt() {
        let mut alloc: DefaultAlloc = DefaultAlloc::new();
        let mut stack: Stack = Stack::new();
        let mut tyck_info_pool: TyckInfoPool = TyckInfoPool::new();

        let mut stack_slice: StackSlice = unsafe { stack.ext_func_call_grow_stack(0, 3, &[]) };

        let str1: Value = Value::new_owned::<String>("114".into());
        let str2: Value = Value::new_owned::<String>("514".into());
        let str3: Value = Value::new_owned::<String>("1919810".into());
        let mut container: TestContainer<String> = TestContainer::new();
        container.inner.elements.push(str1);
        container.inner.elements.push(str2);
        container.inner.elements.push(str3);
        let vt: GenericTypeVT = create_test_container_vt::<String>(&mut tyck_info_pool);

        let container: Value = Value::new_container(
            move_to_heap(Wrapper::new_owned(container)).as_ptr() as *mut Wrapper<()>,
            &vt
        );

        unsafe {
            alloc.add_stack(&stack);
            alloc.add_managed(str1);
            alloc.add_managed(str2);
            alloc.add_managed(str3);
            alloc.add_managed(container);

            stack_slice.set_value(0, container);
            alloc.collect();
            assert!(alloc.contains_ptr(str1.ptr_repr));
            assert!(alloc.contains_ptr(str2.ptr_repr));
            assert!(alloc.contains_ptr(str3.ptr_repr));
            assert!(alloc.contains_ptr(container.ptr_repr));

            stack_slice.set_value(0, str1);
            stack_slice.set_value(1, str2);
            stack_slice.set_value(2, str3);
            alloc.collect();
            assert!(alloc.contains_ptr(str1.ptr_repr));
            assert!(alloc.contains_ptr(str2.ptr_repr));
            assert!(alloc.contains_ptr(str3.ptr_repr));
            assert!(!alloc.contains_ptr(container.ptr_repr));

            stack_slice.set_value(1, Value::new_null());
            alloc.collect();
            assert!(alloc.contains_ptr(str1.ptr_repr));
            assert!(!alloc.contains_ptr(str2.ptr_repr));
            assert!(alloc.contains_ptr(str3.ptr_repr));
            assert!(!alloc.contains_ptr(container.ptr_repr));
        }
    }
}
