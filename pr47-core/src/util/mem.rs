use std::hash::Hash;

#[repr(C)]
#[derive(Clone, Copy, Hash, Debug, Eq, PartialEq)]
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
