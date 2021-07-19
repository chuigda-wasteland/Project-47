use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit, transmute};
use std::ptr::NonNull;

use crate::data::custom_vt::ContainerVT;
use crate::data::traits::StaticBase;
use crate::data::tyck::{ContainerTyckInfo, TyckInfo};
use crate::data::wrapper::{OwnershipInfo, Wrapper};
use crate::util::mem::FatPointer;
use crate::util::void::Void;

pub struct TestContainer<T: 'static> {
    pub elements: Vec<FatPointer>,
    _phantom: PhantomData<T>
}

impl<T: 'static> TestContainer<T>
    where Void: StaticBase<T>
{
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            _phantom: PhantomData::default()
        }
    }
}

impl<T: 'static> StaticBase<TestContainer<T>> for Void
    where Void: StaticBase<T>
{
    fn type_id() -> TypeId {
        TypeId::of::<TestContainer<()>>()
    }

    fn tyck_info() -> TyckInfo {
        let element_tyck_info: TyckInfo = <Void as StaticBase<T>>::tyck_info();
        TyckInfo::Container(ContainerTyckInfo {
            type_id: TypeId::of::<TestContainer<()>>(),
            params: unsafe {
                vec![
                    NonNull::new_unchecked(Box::leak(Box::new(element_tyck_info)))
                ]
            }
        })
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(ContainerTyckInfo { type_id, params }) = tyck_info {
            TypeId::of::<TestContainer<()>>() == *type_id
            && params.len() == 1
            && <Void as StaticBase<T>>::tyck(unsafe { params.get_unchecked(0).as_ref() })
            // TODO: take `any` type into consideration
        } else {
            false
        }
    }

    fn type_name() -> String {
        "TestContainer".into()
    }

    fn children(vself: *const TestContainer<T>)
        -> Option<Box<dyn Iterator<Item=FatPointer> + 'static>>
    {
        let vself: &TestContainer<T> = unsafe { &*vself };
        let iter: Box<dyn Iterator<Item=FatPointer> + '_> =
            Box::new(vself.elements.iter().map(|x: &FatPointer| *x));
        let iter: Box<dyn Iterator<Item=FatPointer> + 'static> = unsafe { transmute::<>(iter) };
        Some(iter)
    }
}

#[cfg(debug_assertions)]
unsafe fn test_container_move_out_ck(this: *mut (), out: *mut (), type_id: TypeId) {
    assert_eq!(type_id, TypeId::of::<TestContainer<()>>());
    let this: &mut Wrapper<TestContainer<()>> = &mut *(this as *mut Wrapper<_>);
    let out: &mut MaybeUninit<TestContainer<()>> = &mut *(out as *mut MaybeUninit<_>);

    assert_eq!(this.ownership_info, OwnershipInfo::VMOwned as u8);
    let test_container: TestContainer<()> = ManuallyDrop::take(&mut this.data.owned).assume_init();
    std::ptr::write(out.as_mut_ptr(), test_container);
    this.ownership_info = OwnershipInfo::MovedToRust as u8;
}

#[cfg(not(debug_assertions))]
unsafe fn test_container_move_out(this: *mut (), out: *mut ()) {
    let this: &mut Wrapper<TestContainer<()>> = &mut *(this as *mut Wrapper<_>);
    let out: &mut MaybeUninit<TestContainer<()>> = &mut *(out as *mut MaybeUninit<_>);

    let test_container: TestContainer<()> = ManuallyDrop::take(&mut this.data.owned).assume_init();
    std::ptr::write(out.as_mut_ptr(), test_container);
    this.ownership_info = OwnershipInfo::MovedToRust as u8;
}

unsafe fn test_container_children(this: *const ()) -> Box<dyn Iterator<Item=FatPointer>> {
    let this: &mut Wrapper<TestContainer<()>> = &mut *(this as *mut Wrapper<_>);
    let iter: Box<dyn Iterator<Item=FatPointer> + '_> =
        Box::new((*(this.data.owned.as_ptr())).elements.iter().map(|x: &FatPointer| *x));
    transmute::<_, Box<dyn Iterator<Item=FatPointer> + 'static>>(iter)
}

unsafe fn test_container_drop(this: *mut()) {
    let this: &mut Wrapper<TestContainer<()>> = &mut *(this as *mut Wrapper<_>);
    let test_container: TestContainer<()> = ManuallyDrop::take(&mut this.data.owned).assume_init();
    drop(test_container)
}

pub fn create_test_container_vt<T: 'static>() -> ContainerVT
    where Void: StaticBase<T>
{
    let elem_tyck_info: TyckInfo = <Void as StaticBase<T>>::tyck_info();
    let tyck_info: ContainerTyckInfo = ContainerTyckInfo {
        type_id: TypeId::of::<TestContainer<()>>(),
        params: vec![ NonNull::new(Box::leak(Box::new(elem_tyck_info))).unwrap() ]
    };

    ContainerVT {
        tyck_info: NonNull::new(Box::leak(Box::new(tyck_info))).unwrap(),
        type_name: "TestContainer".to_string(),
        #[cfg(debug_assertions)]
        move_out_fn: test_container_move_out_ck,
        #[cfg(not(debug_assertions))]
        move_out_fn: test_container_move_out,
        children_fn: test_container_children,
        drop_fn: test_container_drop
    }
}
