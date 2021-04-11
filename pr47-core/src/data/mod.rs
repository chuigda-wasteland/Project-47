pub mod custom_vt;
pub mod traits;
pub mod tyck;
pub mod value_typed;
pub mod wrapper;

use crate::data::value_typed::{ValueTypedData, VALUE_TYPE_MASK};
use crate::data::traits::StaticBase;
use crate::data::wrapper::{DynBase, Wrapper, GcInfo, GC_INFO_MASK};
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

    pub unsafe fn ref_count(&self) -> u32 {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.ptr_repr.ptr as *const u32)
    }

    pub unsafe fn incr_ref_count(&mut self) {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.ptr_repr.ptr as *mut u32) += 1
    }

    pub unsafe fn decr_ref_count(&mut self) {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.ptr_repr.ptr as *mut u32) -= 1
    }

    #[cfg(debug_assertions)]
    fn assert_shared(&self) {
        let gc_info: GcInfo = unsafe { self.gc_info() };
        assert!(gc_info == GcInfo::SharedFromRust
                || gc_info == GcInfo::SharedToRust);
    }

    pub unsafe fn gc_info(&self) -> GcInfo {
        debug_assert_eq!(self.is_ref());
        *(self.ptr_repr.ptr + 4 as *const u8) & GC_INFO_MASK
    }

    pub unsafe fn set_gc_info(&mut self, gc_info: GcInfo) {
        debug_assert_eq!(self.is_ref());
        *(self.ptr_repr.ptr + 4 as *mut u8) = gc_info.into();
    }
}
