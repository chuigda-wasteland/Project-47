#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FatPointer {
    pub ptr: usize,
    pub trivia: usize
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
}
