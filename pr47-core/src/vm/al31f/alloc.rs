use std::collections::{HashSet, VecDeque};
use std::mem::transmute;

use crate::data::PTR_BITS_MASK_USIZE;
use crate::data::custom_vt::{CONTAINER_MASK, ContainerVT};
use crate::data::value_typed::VALUE_TYPE_MASK;
use crate::data::wrapper::{DynBase, Wrapper, OWN_INFO_COLLECT_MASK};
use crate::util::mem::FatPointer;
use crate::vm::al31f::stack::Stack;

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

#[repr(u8)]
pub enum DefaultGCStatus {
    Unmarked = 0,
    Marked = 1
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
        for ptr in self.managed.iter() {
            debug_assert_eq!(ptr.ptr & (VALUE_TYPE_MASK as usize), 0);
            let wrapper: *mut Wrapper<()> = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;
            (*wrapper).gc_info = DefaultGCStatus::Unmarked as u8;
        }

        let mut to_scan: VecDeque<FatPointer> = VecDeque::new();

        for stack /*: &*const Stack*/ in self.stacks.iter() {
            #[cfg(debug_assertions)]
            for stack_value /*: &Option<Value>*/ in &(**stack).values {
                if let Some(stack_value /*: &Value*/) = stack_value {
                    if !stack_value.is_null() && !stack_value.is_value() {
                        to_scan.push_back(stack_value.ptr_repr);
                    }
                }
            }

            #[cfg(not(debug_assertions))]
            for stack_value /*: &Value*/ in &(**stack).values {
                if !stack_value.is_null() && !stack_value.is_value() {
                    to_scan.push_back(stack_value.ptr_repr);
                }
            }
        }

        while !to_scan.is_empty() {
            let ptr: FatPointer = to_scan.pop_front().unwrap();
            let wrapper: *mut Wrapper<()> = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;

            if (*wrapper).gc_info == (DefaultGCStatus::Marked as u8) {
                continue;
            }

            (*wrapper).gc_info = DefaultGCStatus::Marked as u8;
            if ptr.trivia & (CONTAINER_MASK as usize) != 0 {
                let dyn_base: *mut dyn DynBase = transmute::<>(ptr);
                if let Some(children /*: Box<dyn Iterator>*/) = (*dyn_base).children() {
                    for child in children {
                        to_scan.push_back(child);
                    }
                }
            } else {
                let container_vt: *const ContainerVT = ptr.trivia as *const _;
                let ptr: *const () = (ptr.ptr & PTR_BITS_MASK_USIZE) as *const _;
                for child /*: FatPointer*/ in ((*container_vt).children_fn)(ptr) {
                    to_scan.push_back(child);
                }
            }
        }

        let mut to_collect: Vec<FatPointer> = Vec::new();
        for ptr /*: FatPointer*/ in self.managed.iter() {
            let wrapper: *mut Wrapper<()> = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;
            if (*wrapper).ownership_info & OWN_INFO_COLLECT_MASK != 0 {
                to_collect.push(*ptr);
            }
        }

        for ptr /*: FatPointer*/ in to_collect {
            if ptr.ptr & (CONTAINER_MASK as usize) != 0 {
                let container: *mut () = (ptr.ptr & PTR_BITS_MASK_USIZE) as *mut _;
                let vt: *const ContainerVT = ptr.trivia as *const _;
                ((*vt).drop_fn)(container);
            } else {
                let dyn_base: *mut dyn DynBase = transmute::<>(ptr);
                let boxed: Box<dyn DynBase> = Box::from_raw(dyn_base);
                drop(boxed);
            }
            self.managed.remove(&ptr);
        }
    }

    fn set_gc_allowed(&mut self, allowed: bool) {
        self.gc_allowed = allowed;
    }
}
