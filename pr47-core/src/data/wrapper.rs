use std::any::TypeId;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::addr_of;
use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfo;
use crate::util::void::Void;

pub const GC_MARKED_MASK: u8 = 0b1_00_00000;
pub const GC_INFO_MASK: u8   = 0b0_00_11111;

#[repr(u8)]
pub enum GcInfo {
    // R = Read
    // W = Write
    // M = Move
    // D = Delete
    // O = Virtual Machine Owned
    //                    M    R W M D O
    Owned             = 0b0_00_1_1_1_1_1,
    SharedFromRust    = 0b0_00_1_0_0_0_0,
    MutSharedFromRust = 0b0_00_1_1_0_0_0,
    SharedToRust      = 0b0_00_1_0_0_0_1,
    MutSharedToRust   = 0b0_00_0_0_0_0_1,
    MovedToRust       = 0b0_00_0_0_0_1_0
}

impl Into<u8> for GcInfo {
    fn into(self) -> u8 {
        self as u8
    }
}

impl GcInfo {
    pub unsafe fn unsafe_from(input: u8) -> Self {
        std::mem::transmute::<u8, Self>(input)
    }
}

#[repr(C)]
pub union WrapperData<T: 'static> {
    pub ptr: *mut T,
    pub owned: ManuallyDrop<MaybeUninit<T>>
}

#[repr(C, align(8))]
pub struct Wrapper<T: 'static> {
    /* +0 */ pub refcount: u32,
    /* +4 */ pub gc_info: u8,
    /* +5 */ pub data_offset: u8,

    pub data: WrapperData<T>
}

impl<T: 'static> Wrapper<T> {
    pub fn new_owned(data: T) -> Self {
        let mut ret: Wrapper<T> = Self {
            refcount: 0,
            gc_info: GcInfo::Owned.into(),
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
            gc_info: GcInfo::SharedFromRust.into(),
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
            gc_info: GcInfo::MutSharedFromRust.into(),
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
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: null_mut()
            }
        };

        assert_eq!(addr_of!(w.refcount) as usize - addr_of!(w) as usize, 0);
        assert_eq!(addr_of!(w.gc_info) as usize - addr_of!(w) as usize, 4);
        assert_eq!(addr_of!(w.data_offset) as usize - addr_of!(w) as usize, 5);

        let w: Wrapper<()> = Wrapper {
            refcount: 42,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: null_mut()
            }
        };

        assert_eq!(addr_of!(w.refcount) as usize - addr_of!(w) as usize, 0);
        assert_eq!(addr_of!(w.gc_info) as usize - addr_of!(w) as usize, 4);
        assert_eq!(addr_of!(w.data_offset) as usize - addr_of!(w) as usize, 5);

        let w: Wrapper<TestStruct2> = Wrapper {
            refcount: 42,
            gc_info: 0,
            data_offset: 0,
            data: WrapperData {
                ptr: null_mut()
            }
        };

        assert_eq!(addr_of!(w.refcount) as usize - addr_of!(w) as usize, 0);
        assert_eq!(addr_of!(w.gc_info) as usize - addr_of!(w) as usize, 4);
        assert_eq!(addr_of!(w.data_offset) as usize - addr_of!(w) as usize, 5);
    }
}
