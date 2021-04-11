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
        self as u8
    }
}

impl ValueTypeTag {
    pub unsafe fn unsafe_from(input: u8) -> Self {
        std::mem::transmute::<u8, Self>(input)
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

impl From<i64> for ValueTypedData {
    fn from(int_value: i64) -> Self {
        Self {
            tag: ValueTypeTag::Int as u64 | VALUE_TYPE_MASK as u64,
            inner: ValueTypedDataInner {
                int_value
            }
        }
    }
}

impl From<f64> for ValueTypedData {
    fn from(float_value: f64) -> Self {
        Self {
            tag: ValueTypeTag::Float as u64 | VALUE_TYPE_MASK as u64,
            inner: ValueTypedDataInner {
                float_value
            }
        }
    }
}

impl From<char> for ValueTypedData {
    fn from(char_value: char) -> Self {
        Self {
            tag: ValueTypeTag::Char as u64 | VALUE_TYPE_MASK as u64,
            inner: ValueTypedDataInner {
                char_value
            }
        }
    }
}

impl From<bool> for ValueTypedData {
    fn from(bool_value: bool) -> Self {
        Self {
            tag: ValueTypeTag::Bool as u64 | VALUE_TYPE_MASK as u64,
            inner: ValueTypedDataInner {
                bool_value
            }
        }
    }
}
