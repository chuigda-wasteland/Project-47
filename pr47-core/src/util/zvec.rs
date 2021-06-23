use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::intrinsics::copy_nonoverlapping;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::NonNull;
use std::slice;
use std::slice::SliceIndex;

use unchecked_unwrap::UncheckedUnwrap;

pub unsafe trait ZeroInit : Copy {}

struct ZeroRawVec<T: ZeroInit> {
    ptr: NonNull<T>,
    cap: usize
}

impl<T: ZeroInit> ZeroRawVec<T> {
    pub unsafe fn new(cap: usize) -> Self {
        debug_assert_ne!(cap, 0);
        let layout: Layout = Layout::array::<T>(cap).unwrap();
        let ptr: *mut u8 = alloc_zeroed(layout);
        let ptr: NonNull<T> = NonNull::new(ptr as *mut T).unwrap();
        Self { ptr, cap }
    }

    pub fn extend(&mut self, cap: usize) {
        let double_cap: usize = self.cap * 2;
        let new_cap: usize = usize::max(double_cap, cap);

        let old_layout: Layout = unsafe { Layout::array::<T>(self.cap).unchecked_unwrap() };
        let new_layout: Layout = Layout::array::<T>(new_cap).unwrap();
        let old_ptr: *mut u8 = self.ptr.as_ptr() as _;
        let new_ptr: *mut u8 = unsafe { alloc_zeroed(new_layout) };
        self.ptr = NonNull::new(new_ptr as *mut T).unwrap();
        self.cap = new_cap;
        unsafe {
            copy_nonoverlapping(old_ptr, new_ptr, old_layout.size());
            dealloc(old_ptr, old_layout);
        }
    }
}

impl<T: ZeroInit> Drop for ZeroRawVec<T> {
    fn drop(&mut self) {
        let layout: Layout = unsafe { Layout::array::<T>(self.cap).unchecked_unwrap() };
        unsafe { dealloc(self.ptr.as_ptr() as _, layout); }
    }
}

pub struct ZeroVec<T: ZeroInit> {
    raw: ZeroRawVec<T>,
    len: usize
}

impl<T: ZeroInit> ZeroVec<T> {
    pub fn new() -> Self {
        Self {
            raw: unsafe { ZeroRawVec::new(16) },
            len: 0
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        assert_ne!(cap, 0);
        Self {
            raw: unsafe { ZeroRawVec::new(cap) },
            len: 0
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        if new_len > self.len && new_len > self.raw.cap {
            self.raw.extend(new_len);
        }
        self.len = new_len
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get_unchecked(&self, idx: usize) -> &T {
        unsafe { &*self.raw.ptr.as_ptr().add(idx) }
    }

    pub fn get_unchecked_mut(&mut self, idx: usize) -> &mut T {
        unsafe { &mut *self.raw.ptr.as_ptr().add(idx) }
    }
}

impl<T: ZeroInit> Deref for ZeroVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.raw.ptr.as_ptr() as _, self.len) }
    }
}

impl<T: ZeroInit> DerefMut for ZeroVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.raw.ptr.as_ptr(), self.len) }
    }
}

impl<T: ZeroInit, I: SliceIndex<[T]>> Index<I> for ZeroVec<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T: ZeroInit, I: SliceIndex<[T]>> IndexMut<I> for ZeroVec<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}
