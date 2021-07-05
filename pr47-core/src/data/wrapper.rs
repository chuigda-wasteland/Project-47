use std::any::TypeId;
use std::iter::Iterator;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::addr_of;

use unchecked_unwrap::UncheckedUnwrap;

use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfo;
use crate::util::mem::FatPointer;
use crate::util::unsafe_from::UnsafeFrom;
use crate::util::void::Void;

pub const OWN_INFO_READ_MASK: u8    = 0b000_1_0_0_0_0;
pub const OWN_INFO_WRITE_MASK: u8   = 0b000_0_1_0_0_0;
pub const OWN_INFO_MOVE_MASK: u8    = 0b000_0_0_1_0_0;
pub const OWN_INFO_COLLECT_MASK: u8 = 0b000_0_0_0_1_0;
pub const OWN_INFO_OWNED_MASK: u8   = 0b000_0_0_0_0_1;

/// Ownership information
///
/// At one time, a Pr47 heap value may be *owned by the VM*, *shared/mutably shared from Rust*,
/// *shared/mutably shared to Rust* or *moved to Rust* while only having a vacant shell.
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum OwnershipInfo {
    // R = Read
    // W = Write
    // M = Move
    // C = Collectable
    // O = Owned by VM
    //                        R W M C O
    VMOwned           = 0b000_1_1_1_1_1,
    SharedFromRust    = 0b000_1_0_0_1_0,
    MutSharedFromRust = 0b000_1_1_0_1_0,
    SharedToRust      = 0b000_1_0_0_0_1, // also used by global constant objects
    MutSharedToRust   = 0b000_0_0_0_0_1,
    MovedToRust       = 0b000_0_0_0_1_0
}

impl OwnershipInfo {
    #[inline(always)] pub fn is_readable(self) -> bool {
        (self as u8) & OWN_INFO_READ_MASK != 0
    }

    #[inline(always)] pub fn is_writeable(self) -> bool {
        (self as u8) & OWN_INFO_WRITE_MASK != 0
    }

    #[inline(always)] pub fn is_movable(self) -> bool {
        (self as u8) & OWN_INFO_MOVE_MASK != 0
    }

    #[inline(always)] pub fn is_collectable(self) -> bool {
        (self as u8) & OWN_INFO_COLLECT_MASK != 0
    }

    #[inline(always)] pub fn is_owned(self) -> bool {
        (self as u8) & OWN_INFO_OWNED_MASK != 0
    }
}

impl UnsafeFrom<u8> for OwnershipInfo {
    unsafe fn unsafe_from(data: u8) -> Self {
        std::mem::transmute::<u8, Self>(data)
    }
}

/// Internal representation of `Wrapper` data. Can be either
///   * A piece of owned data, represented by a `ManuallyDrop<MaybeUninit<T>>`
///   * A reference to data shared from Rust, represented by a `*mut T`
#[repr(C)]
pub union WrapperData<T: 'static> {
    pub owned: ManuallyDrop<MaybeUninit<T>>,
    pub ptr: *mut T
}

#[repr(C, align(8))]
pub struct Wrapper<T: 'static> {
    /* +0 */ pub refcount: u32,
    /* +4 */ pub ownership_info: u8,
    /* +5 */ pub gc_info: u8,
    /* +6 */ pub data_offset: u8,

    /* +data_offset */ pub data: WrapperData<T>
}

impl<T: 'static> Wrapper<T> {
    pub fn new_owned(data: T) -> Self {
        let mut ret: Wrapper<T> = Self {
            refcount: 0,
            ownership_info: OwnershipInfo::VMOwned as u8,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                owned: ManuallyDrop::new(MaybeUninit::new(data))
            }
        };
        ret.data_offset = (addr_of!(ret.data) as usize - addr_of!(ret) as usize) as u8;
        ret
    }

    pub fn new_ref(ptr: *const T) -> Self {
        let mut ret: Wrapper<T> = Self {
            refcount: 1,
            ownership_info: OwnershipInfo::SharedFromRust as u8,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: ptr as *mut T
            }
        };
        ret.data_offset = (addr_of!(ret.data) as usize - addr_of!(ret) as usize) as u8;
        ret
    }

    pub fn new_mut_ref(ptr: *mut T) -> Self {
        let mut ret: Wrapper<T> = Self {
            refcount: 1,
            ownership_info: OwnershipInfo::MutSharedFromRust as u8,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr
            }
        };
        ret.data_offset = (addr_of!(ret.data) as usize - addr_of!(ret) as usize) as u8;
        ret
    }
}

pub trait DynBase {
    fn dyn_type_id(&self) -> TypeId;

    fn dyn_type_name(&self) -> String;

    fn dyn_tyck(&self, tyck_info: &TyckInfo) -> bool;

    #[cfg(debug_assertions)]
    unsafe fn move_out_ck(&mut self, out: *mut (), type_id: TypeId);

    #[cfg(not(debug_assertions))]
    unsafe fn move_out(&mut self, out: *mut ());

    fn children(&self) -> Option<Box<dyn Iterator<Item=FatPointer>>>;
}

impl<T: 'static> DynBase for Wrapper<T> where Void: StaticBase<T> {
    fn dyn_type_id(&self) -> TypeId {
        <Void as StaticBase<T>>::type_id()
    }

    fn dyn_type_name(&self) -> String {
        <Void as StaticBase<T>>::type_name()
    }

    fn dyn_tyck(&self, tyck_info: &TyckInfo) -> bool {
        <Void as StaticBase<T>>::tyck(tyck_info)
    }

    #[cfg(debug_assertions)]
    unsafe fn move_out_ck(&mut self, out: *mut (), type_id: TypeId) {
        debug_assert_eq!(self.dyn_type_id(), type_id);
        debug_assert!(OwnershipInfo::unsafe_from(self.ownership_info).is_movable());
        let dest: &mut MaybeUninit<T> = (out as *mut MaybeUninit<T>).as_mut().unchecked_unwrap();
        *dest.as_mut_ptr() = ManuallyDrop::take(&mut self.data.owned).assume_init();
        self.ownership_info = OwnershipInfo::MovedToRust as u8;
    }

    #[cfg(not(debug_assertions))]
    unsafe fn move_out(&mut self, out: *mut ()) {
        let dest: &mut MaybeUninit<T>
            = (out as *mut MaybeUninit<T>).as_mut().unchecked_unwrap();
        *dest.as_mut_ptr() = ManuallyDrop::take(&mut self.data.owned).assume_init();
        self.ownership_info = OwnershipInfo::MovedToRust as u8;
    }

    #[inline]
    fn children(&self) -> Option<Box<dyn Iterator<Item=FatPointer>>> {
        debug_assert_ne!(self.ownership_info & OWN_INFO_READ_MASK, 0);
        // TODO implement
        None
    }
}

#[cfg(test)]
mod test {
    use std::ptr::{addr_of, null_mut};
    use crate::data::wrapper::{Wrapper, WrapperData};

    #[allow(dead_code)]
    struct TestStruct {
        field1: i32,
        field2: i64,
        field3: std::string::String
    }

    #[allow(dead_code)]
    #[repr(align(16))]
    struct TestStruct2();

    #[test]
    fn test_mem_layout() {
        let w: Wrapper<TestStruct> = Wrapper {
            refcount: 42,
            ownership_info: 0,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: null_mut()
            }
        };

        assert_eq!(addr_of!(w.refcount) as usize - addr_of!(w) as usize, 0);
        assert_eq!(addr_of!(w.ownership_info) as usize - addr_of!(w) as usize, 4);
        assert_eq!(addr_of!(w.gc_info) as usize - addr_of!(w) as usize, 5);
        assert_eq!(addr_of!(w.data_offset) as usize - addr_of!(w) as usize, 6);

        let w: Wrapper<()> = Wrapper {
            refcount: 42,
            ownership_info: 0,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: null_mut()
            }
        };

        assert_eq!(addr_of!(w.refcount) as usize - addr_of!(w) as usize, 0);
        assert_eq!(addr_of!(w.ownership_info) as usize - addr_of!(w) as usize, 4);
        assert_eq!(addr_of!(w.gc_info) as usize - addr_of!(w) as usize, 5);
        assert_eq!(addr_of!(w.data_offset) as usize - addr_of!(w) as usize, 6);

        let w: Wrapper<TestStruct2> = Wrapper {
            refcount: 42,
            ownership_info: 0,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: null_mut()
            }
        };

        assert_eq!(addr_of!(w.refcount) as usize - addr_of!(w) as usize, 0);
        assert_eq!(addr_of!(w.ownership_info) as usize - addr_of!(w) as usize, 4);
        assert_eq!(addr_of!(w.gc_info) as usize - addr_of!(w) as usize, 5);
        assert_eq!(addr_of!(w.data_offset) as usize - addr_of!(w) as usize, 6);
    }
}
