pub const VALUE_TYPE_MASK: u8     = 0b00_000_001;
pub const VALUE_TYPE_TAG_MASK: u8 = 0b00_111_000;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ValueTypeTag {
    Int   = 0b00_001_000,
    Float = 0b00_010_000,
    Char  = 0b00_011_000,
    Bool  = 0b00_100_000
}

impl Into<u8> for ValueTypeTag {
    fn into(self) -> u8 {
        unsafe { std::mem::transmute::<Self, u8>(self) }
    }
}

impl ValueTypeTag {
    unsafe fn unsafe_from(input: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(input) }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ValueTypedDataInner {
    pub int_value: i64,
    pub float_value: f64,
    pub char_value: char,
    pub bool_value: bool
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ValueTypedData {
    pub tag: u64,
    pub inner: ValueTypedDataInner
}
