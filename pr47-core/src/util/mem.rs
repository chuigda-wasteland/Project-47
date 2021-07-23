use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::util::std_ext::BoxedExt;

#[inline] pub fn move_to_heap<T>(data: T) -> NonNull<T> {
    let boxed: Box<T> = Box::new(data);
    leak_as_nonnull(boxed)
}

#[inline] pub fn leak_as_nonnull<T>(boxed: Box<T>) -> NonNull<T>
    where T: ?Sized
{
    let ptr: *mut T = Box::into_raw(boxed);
    unsafe { NonNull::new_unchecked(ptr) }
}

#[inline] pub unsafe fn reclaim_as_boxed<T>(raw_ptr: NonNull<T>) -> Box<T>
    where T: ?Sized
{
    Box::from_raw(raw_ptr.as_ptr())
}

#[repr(transparent)]
pub struct Korobka<T>(NonNull<T>, PhantomData<T>);

impl<T> Drop for Korobka<T> {
    fn drop(&mut self) {
        let boxed: Box<T> = unsafe { Box::reclaim(self.0) };
        drop(boxed);
    }
}

impl<T> Korobka<T> {
    #[inline(always)] pub fn new(t: T) -> Self {
        Self (Box::new(t).leak_as_nonnull(), PhantomData::default())
    }

    pub fn cast<U>(self) -> Korobka<U> {
        Korobka(self.0.cast::<U>(), PhantomData::default())
    }

    pub const fn as_ptr(&self) -> *const T {
        self.0.as_ptr() as *const _
    }

    pub const fn as_nonnull(&self) -> NonNull<T> {
        self.0
    }
}

impl<T> AsRef<T> for Korobka<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> Borrow<T> for Korobka<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T> Deref for Korobka<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> From<Box<T>> for Korobka<T> {
    fn from(boxed: Box<T>) -> Self {
        Self (boxed.leak_as_nonnull(), PhantomData::default())
    }
}

impl<T> Hash for Korobka<T> where T: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            self.0.as_ref().hash(state);
        }
    }
}

impl<T> PartialEq for Korobka<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref().eq(other.0.as_ref()) }
    }
}

impl<T> Eq for Korobka<T> where T: Eq + PartialEq {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FatPointer {
    pub ptr: usize,
    pub trivia: usize
}

impl FatPointer {
    pub fn new(ptr: usize, trivia: usize) -> Self {
        Self {
            ptr, trivia
        }
    }
}

impl Default for FatPointer {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod test {
    use crate::util::mem::FatPointer;

    #[test]
    fn test_fat_pointer_size() {
        trait UselessTrait {}

        assert_eq!(std::mem::size_of::<FatPointer>(),
                   std::mem::size_of::<*mut dyn UselessTrait>());
        assert_eq!(std::mem::align_of::<FatPointer>(),
                   std::mem::align_of::<*mut dyn UselessTrait>());
    }

    #[test]
    fn test_fat_pointer_layout() {
        trait UselessTrait {}
        struct MeinStrukt();

        impl UselessTrait for MeinStrukt {}

        let s = MeinStrukt();
        let ptr: *const MeinStrukt = &s as *const MeinStrukt;
        let fat_ptr: *const dyn UselessTrait = &s as &dyn UselessTrait as *const dyn UselessTrait;
        let fat_ptr: FatPointer = unsafe { std::mem::transmute::<>(fat_ptr) };

        assert_eq!(fat_ptr.ptr, ptr as usize);
    }

    #[test]
    fn test_fat_pointer_layout2() {
        let slice: &[i32; 4] = &[114, 514, 1919, 810];
        let ptr: *const i32 = &slice[0] as *const i32;
        let fat_ptr: *const [i32] = slice as *const [i32];
        let fat_ptr: FatPointer = unsafe { std::mem::transmute::<>(fat_ptr) };

        assert_eq!(fat_ptr.ptr, ptr as usize);
        assert_eq!(fat_ptr.trivia, 4);
    }
}
