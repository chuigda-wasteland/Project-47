use crate::util::unsafe_from::UnsafeFrom;

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

pub const INT_TYPE_TAG: usize   = ValueTypeTag::Int as usize;
pub const FLOAT_TYPE_TAG: usize = ValueTypeTag::Float as usize;
pub const CHAR_TYPE_TAG: usize  = ValueTypeTag::Char as usize;
pub const BOOL_TYPE_TAG: usize  = ValueTypeTag::Bool as usize;

impl UnsafeFrom<u8> for ValueTypeTag {
    #[inline(always)] unsafe fn unsafe_from(data: u8) -> Self {
        std::mem::transmute::<u8, Self>(data)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ValueTypedDataInner {
    pub int_value: i64,
    pub float_value: f64,
    pub char_value: char,
    pub bool_value: bool,

    pub repr: u64
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ValueTypedData {
    pub tag: usize,
    pub inner: ValueTypedDataInner
}

impl ValueTypedData {
    #[inline(always)] pub fn new_raw(tag: usize, repr: u64) -> Self {
        Self {
            tag: tag | (VALUE_TYPE_MASK as usize),
            inner: ValueTypedDataInner { repr }
        }
    }
}

impl From<i64> for ValueTypedData {
    #[inline(always)] fn from(int_value: i64) -> Self {
        Self {
            tag: ValueTypeTag::Int as usize | VALUE_TYPE_MASK as usize,
            inner: ValueTypedDataInner {
                int_value
            }
        }
    }
}

impl From<f64> for ValueTypedData {
    #[inline(always)] fn from(float_value: f64) -> Self {
        Self {
            tag: ValueTypeTag::Float as usize | VALUE_TYPE_MASK as usize,
            inner: ValueTypedDataInner {
                float_value
            }
        }
    }
}

impl From<char> for ValueTypedData {
    #[inline(always)] fn from(char_value: char) -> Self {
        Self {
            tag: ValueTypeTag::Char as usize | VALUE_TYPE_MASK as usize,
            inner: ValueTypedDataInner {
                char_value
            }
        }
    }
}

impl From<bool> for ValueTypedData {
    #[inline(always)] fn from(bool_value: bool) -> Self {
        Self {
            tag: ValueTypeTag::Bool as usize | VALUE_TYPE_MASK as usize,
            inner: ValueTypedDataInner {
                bool_value
            }
        }
    }
}
