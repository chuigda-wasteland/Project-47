//! # `unchecked_option.rs`: Provides unchecked variant of `std::option::Option`
//! The `UncheckedOption` is provided as an unsafe counterpart to `std::option::Option`, with
//! no checks or guarantees. User must guarantee the correctness on themselves.

#[cfg(debug_assertions)]
pub struct UncheckedOption<T> {
    inner: Option<T>
}

#[cfg(debug_assertions)]
impl<T> UncheckedOption<T> {
    /// Create an `UncheckedOption` containing given value `t`.
    pub fn new(t: T) -> Self {
        Self {
            inner: Some(t)
        }
    }

    /// Create an empty `UncheckedOption`
    pub fn new_none() -> Self {
        Self {
            inner: None
        }
    }

    /// Assuming the `UncheckedOption` containing a value, take out the item stored in
    /// `UncheckedOption`.
    ///
    /// # Safety
    ///
    /// The `UncheckedOption` must really contains a `T`. If not, this function will panic in
    /// debug build, cause undefined behavior in release build.
    pub unsafe fn take(&mut self) -> T {
        self.inner.take().unwrap()
    }

    /// Assuming the `UncheckedOption` containing a value, get an immutable reference to the item
    /// stored in `UncheckedOption`.
    ///
    /// # Safety
    ///
    /// The `UncheckedOption` must really contains a `T`. If not, this function will panic in
    /// debug build, cause undefined behavior in release build.
    pub unsafe fn get_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    /// Assuming the `UncheckedOption` containing a value, get an mutable reference to the item
    /// stored in `UncheckedOption`.
    ///
    /// # Safety
    ///
    /// The `UncheckedOption` must really contains a `T`. If not, this function will panic in
    /// debug build, cause undefined behavior in release build.
    pub unsafe fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    /// Assuming the `UncheckedOption` is empty, put a `T` object into it.
    ///
    /// # Safety
    ///
    /// The `UncheckedOption` must be empty. If not, this function will panic in
    /// debug build, or cause potential resource leaks in release build. This function does not have
    /// UB, but still marked as `unsafe` in order to remind user.
    pub unsafe fn set(&mut self, t: T) {
        let origin: Option<T> = self.inner.replace(t);
        assert!(origin.is_none());
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
