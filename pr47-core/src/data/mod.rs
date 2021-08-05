pub mod custom_vt;
pub mod exception;
pub mod traits;
pub mod tyck;
pub mod value_typed;
pub mod wrapper;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use unchecked_unwrap::UncheckedUnwrap;

use crate::data::custom_vt::{CONTAINER_MASK, ContainerVT};
use crate::data::traits::StaticBase;
use crate::data::value_typed::{VALUE_TYPE_MASK, ValueTypedData};
use crate::data::wrapper::{DynBase, OwnershipInfo, Wrapper};
use crate::util::mem::{FatPointer, move_to_heap};
use crate::util::std_ext::BoxedExt;
use crate::util::unsafe_from::UnsafeFrom;
use crate::util::void::Void;
use crate::util::zvec::ZeroInit;

#[cfg(any(test, feature = "bench"))]
use std::fmt::{Debug, Formatter};
#[cfg(any(test, feature = "bench"))]
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};

pub const TAG_BITS_MASK: u8 = 0b00000_111;
pub const TAG_BITS_MASK_USIZE: usize = TAG_BITS_MASK as usize;

pub const PTR_BITS_MASK: u8 = !TAG_BITS_MASK;
pub const PTR_BITS_MASK_USIZE: usize = !TAG_BITS_MASK_USIZE;

/// A generic stack value of Pr47. A stack value may be
///   * A *value-typed data*, see `pr47::data::value_typed::ValueTypedData`
///   * A *normal reference* to "heap" object, see `pr47::data::wrapper::DynBase`
///   * A *custom pointer* to container objects created in Pr47 VM, see
///     `pr47::data::wrapper::custom_vt::ContainerVT`
///
/// A normal reference may be either *owned*, or *shared/mutably shared from Rust*. Check
/// documentation of `pr47::data::wrapper::Wrapper` for more information. A custom pointer should
/// never be shared from Rust, since it is only used when creating containers from Pr47 VM.
///
/// Pr47 uses tagged pointers to distinguish these three kinds of values:
///
/// ```text
/// +-----------------------------+
/// |          FatPointer         |
/// +--------------+--------------+
/// |      ptr     |    trivia    |
/// +-------|------+--------------+
///         |
///         |
/// +------------------------------+---+---+---+
/// |        PTR-BITS ... 3        | 2 | 1 | 0 |
/// +------------------------------+---+---+---+
/// | 8 byte aligned pointer value | U | C | V |
/// +------------------------------+---+---+---+
/// ```
///
///   * `U`: Unused
///   * `C`: Container
///   * `V`: Value-typed
///
/// Since `pr47::data::wrapper::Wrapper` is 8 byte aligned, it is safe to use such a tagged-pointer
#[repr(C)]
#[derive(Clone, Copy)]
pub union Value {
    pub ptr: *mut dyn DynBase,
    pub ptr_repr: FatPointer,
    pub vt_data: ValueTypedData,
}

impl Value {
    /// Create a new "owned" `Value`
    pub fn new_owned<T>(data: T) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        Self {
            ptr: move_to_heap(Wrapper::new_owned(data)).as_ptr()
        }
    }

    pub fn new_container<T>(data: T, vt: *const ContainerVT) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        let wrapper: Box<Wrapper<T>> = Box::new(Wrapper::new_owned(data));
        let ptr: usize = wrapper.leak_as_nonnull().as_ptr() as usize;
        let ptr: usize = ptr | (CONTAINER_MASK as usize);
        let trivia: usize = vt as _;

        Self {
            ptr_repr: FatPointer {
                ptr, trivia
            }
        }
    }

    /// Create a new "shared" `Value`
    pub fn new_shared<T>(data: &T) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        Self {
            ptr: move_to_heap(Wrapper::new_ref(data as *const T)).as_ptr()
        }
    }

    /// Create a new "mutably shared" `Value`
    pub fn new_mut_shared<T>(data: &mut T) -> Self
        where T: 'static,
              Void: StaticBase<T>
    {
        Self {
            ptr: move_to_heap(Wrapper::new_mut_ref(data as *mut T)).as_ptr()
        }
    }

    /// Create a new integer `Value`
    #[inline(always)] pub fn new_int(int_value: i64) -> Self {
        Self {
            vt_data: ValueTypedData::from(int_value)
        }
    }

    /// Create a new floating point number `Value`
    #[inline(always)] pub fn new_float(float_value: f64) -> Self {
        Self {
            vt_data: ValueTypedData::from(float_value)
        }
    }

    /// Create a new character `Value`
    #[inline(always)] pub fn new_char(char_value: char) -> Self {
        Self {
            vt_data: ValueTypedData::from(char_value)
        }
    }

    /// Create a new boolean `Value`
    #[inline(always)] pub fn new_bool(bool_value: bool) -> Self {
        Self {
            vt_data: ValueTypedData::from(bool_value)
        }
    }

    /// Create a new `null` `Value`
    #[inline(always)] pub fn new_null() -> Self {
        Self {
            ptr_repr: FatPointer::new(0, 0)
        }
    }

    /// Check if a `Value` is `null`.
    pub fn is_null(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr == 0 && self.ptr_repr.trivia == 0
        }
    }

    /// Check if a `Value` is value-typed
    pub fn is_value(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr & (VALUE_TYPE_MASK as usize) != 0
        }
    }

    /// Check if a `Value` is reference-typed
    pub fn is_ref(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr & (VALUE_TYPE_MASK as usize) == 0
        }
    }

    /// Check if a `Value` is a custom pointer
    pub fn is_container(&self) -> bool {
        unsafe {
            self.ptr_repr.ptr & (CONTAINER_MASK as usize) != 0
        }
    }

    /// Assuming that `self` may be a custom pointer, get the untagged pointer
    #[inline(always)] pub unsafe fn untagged_ptr_field(&self) -> usize {
        self.ptr_repr.ptr & !TAG_BITS_MASK_USIZE
    }

    /// Assuming that `self` **MUST NOT** be a custom pointer, get the fat pointer
    /// `*mut dyn DynBase`
    #[inline(always)] pub unsafe fn untagged_dyn_base(&self) -> *mut dyn DynBase {
        debug_assert_eq!(self.ptr_repr.ptr & CONTAINER_MASK as usize, 0);
        debug_assert!(!self.is_container());
        std::mem::transmute::<FatPointer, *mut dyn DynBase>(
            FatPointer::new(self.ptr_repr.ptr, self.ptr_repr.trivia)
        )
    }

    /// Assuming that `self` may be a custom pointer, get the reference counting
    pub unsafe fn ref_count(&self) -> u32 {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.untagged_ptr_field() as *const u32)
    }

    /// Given that `self` **MUST NOT** be a custom pointer, get the reference counting
    pub unsafe fn ref_count_norm(&self) -> u32 {
        #[cfg(debug_assertions)] self.assert_shared();
        debug_assert!(!self.is_container());
        *(self.ptr_repr.ptr as *const u32)
    }

    /// Assuming that `self` may be a custom pointer, increase the reference counting
    pub unsafe fn incr_ref_count(&self) {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.untagged_ptr_field() as *mut u32) += 1
    }

    /// Given that `self` **MUST NOT** be a custom pointer, increase the reference counting
    pub unsafe fn incr_ref_count_norm(&self) {
        #[cfg(debug_assertions)] self.assert_shared();
        debug_assert!(!self.is_container());
        *(self.ptr_repr.ptr as *mut u32) += 1
    }

    /// Assuming that `self` may be a custom pointer, decrease the reference counting
    pub unsafe fn decr_ref_count(&self) {
        #[cfg(debug_assertions)] self.assert_shared();
        *(self.untagged_ptr_field() as *mut u32) -= 1
    }

    /// Given that `self` **MUST NOT** be a custom pointer, decrease the reference counting
    pub unsafe fn decr_ref_count_norm(&self) {
        #[cfg(debug_assertions)] self.assert_shared();
        debug_assert!(!self.is_container());
        *(self.ptr_repr.ptr as *mut u32) -= 1
    }

    /// Assert `self` to be in a shared status, thus the reference-counting field of `self`
    /// is valid.
    #[cfg(debug_assertions)]
    fn assert_shared(&self) {
        let ownership_info: OwnershipInfo = unsafe { self.ownership_info() };
        assert!(ownership_info == OwnershipInfo::SharedFromRust
                || ownership_info == OwnershipInfo::SharedToRust);
    }

    /// Given that `self` **MUST** be a reference, assuming that `self` may be a custom pointer, get
    /// the ownership info
    pub unsafe fn ownership_info(&self) -> OwnershipInfo {
        debug_assert!(self.is_ref());
        UnsafeFrom::unsafe_from(*((self.untagged_ptr_field() + 4usize) as *const u8))
    }

    /// Given that `self` **MUST** be a reference and **MUST NOT** be a custom pointer, get the
    /// ownership info
    pub unsafe fn ownership_info_norm(&self) -> OwnershipInfo {
        debug_assert!(self.is_ref());
        debug_assert!(!self.is_container());
        UnsafeFrom::unsafe_from(*((self.ptr_repr.ptr + 4usize) as *const u8))
    }

    /// Given that `self` **MUST** be a reference, assuming that `self` may be a custom pointer,
    /// set the ownership info
    pub unsafe fn set_ownership_info(&self, ownership_info: OwnershipInfo) {
        debug_assert!(self.is_ref());
        *((self.untagged_ptr_field() + 4usize) as *mut u8) = ownership_info as u8;
    }

    /// Given that `self` **MUST** be a reference and **MUST NOT** be a custom pointer, set the
    /// ownership info
    pub unsafe fn set_ownership_info_norm(&self, ownership_info: OwnershipInfo) {
        debug_assert!(self.is_ref());
        debug_assert!(!self.is_container());
        *((self.ptr_repr.ptr + 4usize) as *mut u8) = ownership_info as u8;
    }

    /// Given that `self` **MUST** be a reference, assuming that `self` may be a custom pointer,
    /// get the GC information
    pub unsafe fn gc_info(&self) -> u8 {
        debug_assert!(self.is_ref());
        *((self.untagged_ptr_field() + 5usize) as *mut u8)
    }

    /// Given that `self` **MUST** be a reference and **MUST NOT** be a custom pointer, get the
    /// GC information
    pub unsafe fn gc_info_norm(&self) -> u8 {
        debug_assert!(self.is_ref());
        debug_assert!(!self.is_container());
        *((self.ptr_repr.ptr + 5usize) as *mut u8)
    }

    /// Given that `self` **MUST** be a reference, assuming that `self` may be a custom pointer,
    /// set the GC information
    pub unsafe fn set_gc_info(&self, gc_info: u8) {
        debug_assert!(self.is_ref());
        *((self.untagged_ptr_field() + 5usize) as *mut u8) = gc_info;
    }

    /// Given that `self` **MUST** be a reference and **MUST BOT** be a custom pointer, set the GC
    /// information
    pub unsafe fn set_gc_info_norm(&self, gc_info: u8) {
        debug_assert!(self.is_ref());
        debug_assert!(!self.is_container());
        *((self.ptr_repr.ptr + 5usize) as *mut u8) = gc_info;
    }

    /// Given that `self` **MUST** be a reference, assuming that `self` may be a custom pointer,
    /// get a pointer to the referenced data
    pub unsafe fn get_as_mut_ptr<T>(&self) -> *mut T
        where T: 'static,
              Void: StaticBase<T>
    {
        debug_assert!(self.ownership_info().is_readable());
        let data_offset: usize = *((self.untagged_ptr_field() + 6usize) as *mut u8) as usize;
        if self.ownership_info().is_owned() {
            (self.untagged_ptr_field() + data_offset as usize) as *mut T
        } else {
            let ptr: *const *mut T = (self.untagged_ptr_field() + data_offset) as *const *mut T;
            *ptr
        }
    }

    /// Given that `self` **MUST** be a reference and **MUST BOT** be a custom pointer, get a
    /// pointer to the referenced data
    pub unsafe fn get_as_mut_ptr_norm<T>(&self) -> *mut T
        where T: 'static,
              Void: StaticBase<T>
    {
        debug_assert!(self.ownership_info().is_readable());
        let data_offset: usize = *((self.ptr_repr.ptr + 6usize) as *mut u8) as usize;
        if self.ownership_info_norm().is_owned() {
            (self.ptr_repr.ptr + data_offset as usize) as *mut T
        } else {
            let ptr: *const *mut T = (self.ptr_repr.ptr + data_offset) as *const *mut T;
            *ptr
        }
    }

    /// Given that `self` **MUST** be a reference to `T` typed VM-owned data, assuming that `self`
    /// may be a custom pointer, move the referenced data out
    pub unsafe fn move_out<T>(&self) -> T
        where T: 'static,
              Void: StaticBase<T>
    {
        debug_assert!(self.is_ref());
        let mut maybe_uninit: MaybeUninit<T> = MaybeUninit::uninit();
        if !self.is_container() {
            let dyn_base: *mut dyn DynBase = self.ptr;
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

    /// Given that `self` **MUST** be a reference to `T` typed VM-owned data, and **MUST NOT** be a
    /// custom pointer, move out the referenced data out
    pub unsafe fn move_out_norm<T>(&self) -> T
        where T: 'static,
              Void: StaticBase<T>
    {
        debug_assert!(self.is_ref());
        debug_assert!(!self.is_container());
        let mut maybe_uninit: MaybeUninit<T> = MaybeUninit::uninit();
        let dyn_base: *mut dyn DynBase = self.ptr;
        #[cfg(debug_assertions)]
        dyn_base.as_mut().unchecked_unwrap().move_out_ck(
            &mut maybe_uninit as *mut _ as *mut (),
            <Void as StaticBase<T>>::type_id()
        );
        #[cfg(not(debug_assertions))]
            dyn_base.as_mut().unchecked_unwrap().move_out(
            &mut maybe_uninit as *mut _ as *mut ()
        );
        maybe_uninit.assume_init()
    }
}

#[cfg(any(test, feature = "bench"))]
impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_value() {
            unsafe {
                match ValueTypeTag::unsafe_from((self.vt_data.tag as u8) & VALUE_TYPE_TAG_MASK) {
                    ValueTypeTag::Int => write!(f, "IntV({})", self.vt_data.inner.int_value),
                    ValueTypeTag::Float => write!(f, "FloatV({})", self.vt_data.inner.float_value),
                    ValueTypeTag::Char => write!(f, "CharV('{}')", self.vt_data.inner.char_value),
                    ValueTypeTag::Bool => write!(f, "BoolV({})", self.vt_data.inner.bool_value)
                }
            }
        } else if self.is_container() {
            unsafe {
                write!(f, "CustomContainer(ptr = {:X}, vt = {:X})",
                       self.ptr_repr.trivia, self.ptr_repr.ptr)
            }
        } else if self.is_null() {
            write!(f, "Null")
        } else {
            unsafe {
                write!(f, "Reference(ptr = {:X})", self.ptr_repr.ptr)
            }
        }
    }
}

unsafe impl ZeroInit for Value {}

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

#[cfg(test)]
mod test;
