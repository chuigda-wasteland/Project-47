use std::any::TypeId;
use std::mem::{ManuallyDrop, MaybeUninit};
use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfo;
use crate::util::void::Void;

#[repr(C)]
pub union WrapperData<T> {
    pub ptr: *mut T,
    pub value: ManuallyDrop<MaybeUninit<T>>
}

#[repr(C, align(8))]
pub struct Wrapper<T> {
    pub refcount: u32,
    pub gc_info: u8,
    pub data_offset: u8,

    pub data: WrapperData<T>
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
