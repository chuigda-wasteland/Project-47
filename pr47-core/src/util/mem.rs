use std::hash::{Hash, Hasher};
use std::ptr::NonNull;

#[inline] pub fn move_to_heap<T>(data: T) -> NonNull<T> {
    let boxed: Box<T> = Box::new(data);
    leak_as_ptr(boxed)
}

#[inline] pub fn leak_as_ptr<T>(boxed: Box<T>) -> NonNull<T>
    where T: ?Sized
{
    let leaked: &'_ mut T = Box::leak(boxed);
    let ptr: *mut T = leaked as *mut T;
    unsafe { NonNull::new_unchecked(ptr) }
}

#[inline] pub unsafe fn reclaim_as_boxed<T>(raw_ptr: NonNull<T>) -> Box<T>
    where T: ?Sized
{
    Box::from_raw(raw_ptr.as_ptr())
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Ptr<T>(NonNull<T>);

impl<T> Ptr<T> {
    pub const fn new(raw_ptr: NonNull<T>) -> Self {
        Self(raw_ptr)
    }

    pub const unsafe fn new_unchecked(raw_ptr: *mut T) -> Self {
        Self(NonNull::new_unchecked(raw_ptr))
    }

    pub const fn cast<U>(self) -> Ptr<U> {
        Ptr(self.0.cast::<U>())
    }

    pub const fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    pub unsafe fn as_ref<'a>(self) -> &'a T {
        self.0.as_ref()
    }

    pub unsafe fn as_mut<'a>(mut self) -> &'a mut T {
        self.0.as_mut()
    }
}

impl<T> Hash for Ptr<T> where T: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            self.0.as_ref().hash(state);
        }
    }
}

impl<T> PartialEq for Ptr<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref().eq(other.0.as_ref()) }
    }
}

impl<T> Eq for Ptr<T> where T: Eq + PartialEq {}

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
