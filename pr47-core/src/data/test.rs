use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::{addr_of, null_mut};

use crate::data::Value;
use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfo;
use crate::data::wrapper::{Wrapper, WrapperData, DynBase, OwnershipInfo};
use crate::util::mem::FatPointer;
use crate::util::void::Void;

#[allow(dead_code)]
struct TestStruct {
    field1: i32,
    field2: i64,
    field3: std::string::String
}

const TEST_STRUCT_NAME: &'static str = "pr47_core__data__test__TestStruct";

impl StaticBase<TestStruct> for Void {
    fn type_name() -> String {
        TEST_STRUCT_NAME.into()
    }
}

#[allow(dead_code)]
#[repr(align(16))]
struct TestStruct2();

const TEST_STRUCT_NAME2: &'static str = "pr47_core__data__test__TestStruct2";

impl StaticBase<TestStruct2> for Void {
    fn type_name() -> String {
        TEST_STRUCT_NAME2.into()
    }
}

/// Ensure correct memory layout
#[test] fn test_mem_layout() {
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

/// Ensure the wrapper-related functions work properly
#[test] fn test_dyn_base_assoc() {
    let w: Wrapper<TestStruct> = Wrapper::new_owned(TestStruct {
        field1: 114, field2: 514, field3: "1919810".to_string()
    });

    let mut dyn_base: Box<dyn DynBase> = Box::new(w);
    assert_eq!(dyn_base.dyn_type_id(), TypeId::of::<TestStruct>());
    assert_eq!(dyn_base.dyn_type_name(), TEST_STRUCT_NAME);

    let tyck_info: TyckInfo = <Void as StaticBase<TestStruct>>::tyck_info();
    assert!(dyn_base.dyn_tyck(&tyck_info));

    let children: Option<Box<dyn Iterator<Item=FatPointer>>> = dyn_base.children();
    assert!(children.is_none());

    let mut out: MaybeUninit<TestStruct> = MaybeUninit::uninit();
    unsafe { dyn_base.move_out_ck(&mut out as *mut _ as *mut (), TypeId::of::<TestStruct>()); }
    let out: TestStruct = unsafe { out.assume_init() };
    assert_eq!(out.field1, 114);
    assert_eq!(out.field2, 514);
    assert_eq!(out.field3, "1919810");
}

/// Ensure the value-related functions work properly
#[test] fn test_value_assoc_ref() {
    let v: Value = Value::new_owned(TestStruct {
        field1: 114, field2: 514, field3: "1919810".to_string()
    });

    //noinspection RsDropRef
    fn test_once(v: &Value) {
        assert!(v.is_ref());
        assert!(!v.is_value());
        assert!(!v.is_null());
        assert!(!v.is_container());

        unsafe {
            let dyn_base: *mut dyn DynBase = v.untagged_dyn_base();
            let dyn_base: &dyn DynBase = dyn_base.as_ref().unwrap();

            assert_eq!(dyn_base.dyn_type_id(), TypeId::of::<TestStruct>());
            assert_eq!(dyn_base.dyn_type_name(), TEST_STRUCT_NAME);
            let tyck_info: TyckInfo = <Void as StaticBase<TestStruct>>::tyck_info();
            assert!(dyn_base.dyn_tyck(&tyck_info));
            let children: Option<Box<dyn Iterator<Item=FatPointer>>> = dyn_base.children();
            assert!(children.is_none());

            drop(dyn_base);

            let raw_ptr: usize = v.untagged_ptr_field();
            let raw_ptr: *const Wrapper<TestStruct> = raw_ptr as *const _;
            let test_struct_ref: &Wrapper<TestStruct> = raw_ptr.as_ref().unwrap();
            // TODO
            // assert_eq!(test_struct_ref.data.owned.field1, 114);
            // assert_eq!(test_struct_ref.data.owned.field2, 514);
            // assert_eq!(test_struct_ref.data.owned.field3, "1919810");

            drop(test_struct_ref);

            assert_eq!(v.ownership_info(), OwnershipInfo::VMOwned);
            assert_eq!(v.ownership_info_norm(), OwnershipInfo::VMOwned);

            v.set_ownership_info(OwnershipInfo::MutSharedToRust);
            assert_eq!(v.ownership_info(), OwnershipInfo::MutSharedToRust);
            assert_eq!(v.ownership_info_norm(), OwnershipInfo::MutSharedToRust);

            v.set_ownership_info_norm(OwnershipInfo::SharedToRust);
            assert_eq!(v.ownership_info(), OwnershipInfo::SharedToRust);
            assert_eq!(v.ownership_info_norm(), OwnershipInfo::SharedToRust);

            assert_eq!(v.ref_count(), 0);
            assert_eq!(v.ref_count_norm(), 0);

            v.incr_ref_count();
            assert_eq!(v.ref_count(), 1);
            assert_eq!(v.ref_count_norm(), 1);

            v.incr_ref_count_norm();
            assert_eq!(v.ref_count(), 2);
            assert_eq!(v.ref_count_norm(), 2);

            v.decr_ref_count();
            assert_eq!(v.ref_count(), 1);
            assert_eq!(v.ref_count_norm(), 1);

            v.decr_ref_count_norm();
            assert_eq!(v.ref_count(), 0);
            assert_eq!(v.ref_count_norm(), 0);

            v.set_ownership_info(OwnershipInfo::VMOwned);
            assert_eq!(v.ownership_info_norm(), OwnershipInfo::VMOwned);

            assert_eq!(v.gc_info(), 0);
            assert_eq!(v.gc_info_norm(), 0);

            v.set_gc_info(114);
            assert_eq!(v.gc_info(), 114);
            assert_eq!(v.gc_info_norm(), 114);

            v.set_gc_info_norm(115);
            assert_eq!(v.gc_info(), 115);
            assert_eq!(v.gc_info_norm(), 115);

            v.set_gc_info(0);
            assert_eq!(v.gc_info(), 0);
            assert_eq!(v.gc_info_norm(), 0);
        }
    }

    test_once(&v);
    test_once(&v);

    unsafe {
        let test_struct: TestStruct = v.move_out_norm();
        assert_eq!(test_struct.field1, 114);
        assert_eq!(test_struct.field2, 514);
        assert_eq!(test_struct.field3, "1919810");

        assert_eq!(v.ownership_info(), OwnershipInfo::MovedToRust);
        assert_eq!(v.ownership_info_norm(), OwnershipInfo::MovedToRust);

        let dyn_base: Box<dyn DynBase> = Box::from_raw(v.ptr);
        drop(dyn_base);
    }
}

#[test] fn test_value_assoc_val() {
    let v: Value = Value::new_int(114514);
}
