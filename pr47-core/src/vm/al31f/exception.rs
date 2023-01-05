use std::marker::PhantomPinned;
use xjbutil::void::Void;

use crate::data::exception::{CheckedException, ExceptionInner, StackTrace, UncheckedException};
use crate::data::traits::{ChildrenType, StaticBase};

pub struct Exception {
    pub inner: ExceptionInner,
    pub trace: Vec<StackTrace>,
    _pin: PhantomPinned
}

impl Exception {
    #[inline(never)] pub fn checked_exc(checked: CheckedException) -> Self {
        Self {
            inner: ExceptionInner::Checked(checked),
            trace: vec![],
            _pin: PhantomPinned
        }
    }

    #[inline(never)] pub fn unchecked_exc(unchecked: UncheckedException) -> Self {
        Self {
            inner: ExceptionInner::Unchecked(unchecked),
            trace: vec![],
            _pin: PhantomPinned
        }
    }

    pub fn push_stack_trace(&mut self, func_id: usize, insc_ptr: usize) {
        self.trace.push(StackTrace::new(func_id, insc_ptr))
    }

    #[cfg(test)]
    pub fn assert_checked(&self) -> CheckedException {
        match &self.inner {
            ExceptionInner::Checked(e) => e.clone(),
            ExceptionInner::Unchecked(_) => panic!()
        }
    }
}

impl StaticBase<Exception> for Void {
    fn type_name() -> String { "Exception".into() }

    fn children(vself: *const Exception) -> ChildrenType {
        match unsafe { &(*vself).inner } {
            ExceptionInner::Unchecked(_) => None,
            ExceptionInner::Checked(checked) => {
                Some(Box::new(std::iter::once(*checked)))
            }
        }
    }
}
