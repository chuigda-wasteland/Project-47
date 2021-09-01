use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit, transmute};
use std::ptr::NonNull;

use crate::data::custom_vt::ContainerVT;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::tyck::{ContainerTyckInfo, TyckInfo, TyckInfoPool};
use crate::data::wrapper::{OwnershipInfo, Wrapper, DynBase};
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

    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        let elem_tyck_info: NonNull<TyckInfo> = <Void as StaticBase<T>>::tyck_info(tyck_info_pool);
        tyck_info_pool.create_container_type(
            TypeId::of::<TestContainer<()>>(),
            &[elem_tyck_info]
        )
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(ContainerTyckInfo { type_id, params }) = tyck_info {
            let params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
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

    fn children(vself: *const TestContainer<T>) -> ChildrenType {
        let vself: &TestContainer<T> = unsafe { &*vself };
        let iter: Box<dyn Iterator<Item=FatPointer> + '_> =
            Box::new(vself.elements.iter().map(|x: &FatPointer| *x));
        let iter: Box<dyn Iterator<Item=FatPointer> + 'static> = unsafe { transmute::<>(iter) };
        Some(iter)
    }
}

pub fn create_test_container_vt<T: 'static>(tyck_info_pool: &mut TyckInfoPool) -> ContainerVT
    where Void: StaticBase<T>
{
    use crate::data::custom_vt::gen_impls;

    #[cfg(debug_assertions)]
    unsafe fn move_out_ck(this: *mut (), out: *mut (), type_id: TypeId) {
        gen_impls::generic_move_out_ck::<TestContainer<()>>(this, out, type_id)
    }

    #[cfg(not(debug_assertions))]
    unsafe fn move_out(this: *mut (), out: *mut ()) {
        gen_impls::generic_move_out::<TestContainer<()>>(this, out)
    }

    unsafe fn children(this: *const ()) -> ChildrenType {
        gen_impls::generic_children::<TestContainer<()>>(this)
    }

    let elem_tyck_info: NonNull<TyckInfo> = <Void as StaticBase<T>>::tyck_info(tyck_info_pool);
    let tyck_info: NonNull<TyckInfo> =
        tyck_info_pool.create_container_type(TypeId::of::<TestContainer<()>>(), &[elem_tyck_info]);
    let container_tyck_info: NonNull<ContainerTyckInfo> =
        unsafe { tyck_info.as_ref().get_container_tyck_info_unchecked() };

    ContainerVT {
        tyck_info: container_tyck_info,
        type_name: "TestContainer".to_string(),
        #[cfg(debug_assertions)]
        move_out_fn: move_out_ck,
        #[cfg(not(debug_assertions))]
        move_out_fn: move_out,
        children_fn: test_container_children,
        drop_fn: test_container_drop
    }
}
