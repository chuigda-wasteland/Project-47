#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FatPointer {
    pub ptr: usize,
    pub trivia: usize
}
