pub mod custom_vt;
pub mod traits;
pub mod tyck;
pub mod value_typed;
pub mod wrapper;

use std::mem::MaybeUninit;

use crate::data::custom_vt::{CONTAINER_MASK, ContainerVT};
use crate::data::traits::{StaticBase, VMType};
use crate::data::value_typed::{VALUE_TYPE_MASK, ValueTypedData};
use crate::data::wrapper::{GC_INFO_MASK, DynBase, GcInfo, Wrapper};
use crate::util::mem::FatPointer;
use crate::util::unsafe_from::UnsafeFrom;
use crate::util::void::Void;

use unchecked_unwrap::UncheckedUnwrap;
use std::marker::PhantomData;

pub const TAG_BITS_MASK: u8 = 0b00000_111;
pub const TAG_BITS_MASK_USIZE: usize = TAG_BITS_MASK as usize;

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

    pub fn is_container(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr & (CONTAINER_MASK as usize) == 0
        }
    }

    #[inline(always)] pub unsafe fn untagged_ptr_field(&self) -> usize {
        debug_assert_eq!(self.ptr_repr.ptr & CONTAINER_MASK as usize, 0);
        self.ptr_repr.ptr & !TAG_BITS_MASK_USIZE
    }

    #[inline(always)] pub unsafe fn untagged_dyn_base(&self) -> *mut dyn DynBase {
        debug_assert_eq!(self.ptr_repr.ptr & CONTAINER_MASK as usize, 0);
        std::mem::transmute::<FatPointer, *mut dyn DynBase>(
            FatPointer::new(self.ptr_repr.ptr & !TAG_BITS_MASK_USIZE, self.ptr_repr.trivia)
        )
    }

    pub unsafe fn ref_count(&self) -> u32 {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.untagged_ptr_field() as *const u32)
    }

    pub unsafe fn incr_ref_count(&mut self) {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.untagged_ptr_field() as *mut u32) += 1
    }

    pub unsafe fn decr_ref_count(&mut self) {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.untagged_ptr_field() as *mut u32) -= 1
    }

    #[cfg(debug_assertions)]
    fn assert_shared(&self) {
        let gc_info: GcInfo = unsafe { self.gc_info() };
        assert!(gc_info == GcInfo::SharedFromRust
                || gc_info == GcInfo::SharedToRust);
    }

    pub unsafe fn gc_info(&self) -> GcInfo {
        debug_assert!(self.is_ref());
        UnsafeFrom::unsafe_from(*((self.untagged_ptr_field() + 4usize) as *const u8) & GC_INFO_MASK)
    }

    pub unsafe fn set_gc_info(&mut self, gc_info: GcInfo) {
        debug_assert!(self.is_ref());
        *((self.untagged_ptr_field() + 4usize) as *mut u8) = gc_info as u8;
    }

    pub unsafe fn get_as_mut_ptr<T>(&self) -> *mut T
        where T: 'static,
              Void: StaticBase<T>
    {
        debug_assert!(self.gc_info().is_readable());
        let data_offset: usize = *((self.untagged_ptr_field() + 5usize) as *mut u8) as usize;
        if self.gc_info().is_owned() {
            (self.untagged_ptr_field() + data_offset as usize) as *mut T
        } else {
            let ptr: *const *mut T = (self.untagged_ptr_field() + data_offset) as *const *mut T;
            *ptr
        }
    }

    pub unsafe fn move_out<T>(&self) -> T
        where T: 'static,
              Void: StaticBase<T>
    {
        debug_assert!(self.is_ref());
        let mut maybe_uninit: MaybeUninit<T> = MaybeUninit::uninit();
        if self.is_container() {
            let dyn_base: *mut dyn DynBase = self.untagged_dyn_base();
            #[cfg(debug_assertions)]
            dyn_base.as_mut().unchecked_unwrap().move_out_ck(
                &mut maybe_uninit as *mut _ as *mut (),
                <Void as StaticBase<T>>::type_id()
            );
            #[cfg(not(debug_assertions))]
            dyn_base.as_mut().unchecked_unwrap().move_out(
                &mut maybe_uninit as *mut _ as *mut ()
            );
        } else {
            let this_ptr: *mut () = self.untagged_ptr_field() as *mut ();
            let custom_vt: *const ContainerVT = self.ptr_repr.trivia as *const _;

            #[cfg(debug_assertions)]
            (custom_vt.as_ref().unchecked_unwrap().move_out_fn) (
                this_ptr,
                &mut maybe_uninit as *mut _ as *mut (),
                <Void as StaticBase<T>>::type_id()
            );
            #[cfg(not(debug_assertions))]
            (custom_vt.as_ref().unchecked_unwrap().move_out_fn) (
                this_ptr,
                &mut maybe_uninit as *mut _ as *mut (),
            );
        }
        maybe_uninit.assume_init()
    }
}

#[repr(transparent)]
pub struct TypedValue<T: 'static> {
    pub inner: Value,
    _phantom: PhantomData<T>
}

impl<T> TypedValue<T>
    where T: 'static,
          Void: StaticBase<T>
{
    // TODO
}

impl TypedValue<i64> {
    // TODO
}

impl TypedValue<f64> {
    // TODO
}

impl TypedValue<char> {
    // TODO
}

impl TypedValue<bool> {
    // TODO
}
