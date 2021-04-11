pub mod custom_vt;
pub mod traits;
pub mod tyck;
pub mod value_typed;
pub mod wrapper;

use crate::data::value_typed::{ValueTypedData, VALUE_TYPE_MASK};
use crate::data::traits::StaticBase;
use crate::data::wrapper::{DynBase, Wrapper, GcInfo};
use crate::util::mem::FatPointer;
use crate::util::void::Void;

#[repr(C)]
#[derive(Clone, Copy)]
pub union Value {
    pub ptr: *mut dyn DynBase,
    pub ptr_repr: FatPointer,
    pub vt_data: ValueTypedData,
}

impl Value {
    pub fn new_owned<T>(data: T) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        Self {
            ptr: Box::leak(Box::new(Wrapper::new_owned(data)))
        }
    }

    pub fn new_shared<T>(data: &T) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        Self {
            ptr: Box::leak(Box::new(Wrapper::new_ref(data as *const T)))
        }
    }

    pub fn new_mut_shared<T>(data: &mut T) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        Self {
            ptr: Box::leak(Box::new(Wrapper::new_mut_ref(data as *mut T)))
        }
    }

    pub fn new_int(int_value: i64) -> Self {
        Self {
            vt_data: ValueTypedData::from(int_value)
        }
    }

    pub fn new_float(float_value: f64) -> Self {
        Self {
            vt_data: ValueTypedData::from(float_value)
        }
    }

    pub fn new_char(char_value: char) -> Self {
        Self {
            vt_data: ValueTypedData::from(char_value)
        }
    }

    pub fn new_bool(bool_value: bool) -> Self {
        Self {
            vt_data: ValueTypedData::from(bool_value)
        }
    }

    pub fn new_null() -> Self {
        Self {
            ptr_repr: FatPointer::new(0, 0)
        }
    }

    pub fn is_null(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr == 0 && self.ptr_repr.trivia == 0
        }

    }

    pub fn is_value(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr & (VALUE_TYPE_MASK as usize) != 0
        }
    }

    pub fn is_ref(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr & (VALUE_TYPE_MASK as usize) == 0
        }
    }

    pub fn gc_info(&self) -> GcInfo {
        todo!("I forgot how to write pointer offset operation")
    }
}
