#[cfg(debug_assertions)]
pub struct UncheckedOption<T> {
    inner: Option<T>
}

#[cfg(debug_assertions)]
impl<T> UncheckedOption<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: Some(t)
        }
    }

    pub fn new_none() -> Self {
        Self {
            inner: None
        }
    }

    pub unsafe fn take(&mut self) -> T {
        self.inner.take().unwrap()
    }

    pub unsafe fn get_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    pub unsafe fn set(&mut self, t: T) {
        let _ = self.inner.replace(t);
    }
}

#[cfg(not(debug_assertions))]
use std::mem::{MaybeUninit, replace};

#[cfg(not(debug_assertions))]
pub struct UncheckedOption<T> {
    inner: MaybeUninit<T>
}

#[cfg(not(debug_assertions))]
impl<T> UncheckedOption<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: MaybeUninit::new(t)
        }
    }

    pub fn new_none() -> Self {
        Self {
            inner: MaybeUninit::uninit()
        }
    }

    pub unsafe fn take(&mut self) -> T {
        let ret: MaybeUninit<T> = replace(&mut self.inner, MaybeUninit::uninit());
        ret.assume_init()
    }

    pub unsafe fn get_ref(&self) -> &T {
        &*self.inner.as_ptr()
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut *self.inner.as_mut_ptr()
    }

    pub unsafe fn set(&mut self, t: T) {
        let _ = replace(&mut self.inner, MaybeUninit::new(t));
    }
}
