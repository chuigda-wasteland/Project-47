use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::transmute;
use std::ptr::NonNull;

use xjbutil::void::Void;

use crate::data::Value;
use crate::data::generic::GenericTypeVT;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::tyck::{ContainerTyckInfo, TyckInfo, TyckInfoPool};

pub struct GenericTestContainer {
    pub elements: Vec<Value>,
}

impl GenericTestContainer {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }
}

impl StaticBase<GenericTestContainer> for Void {
    fn type_id() -> TypeId {
        TypeId::of::<GenericTestContainer>()
    }

    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        tyck_info_pool.create_container_type(TypeId::of::<GenericTestContainer>(), &[])
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(ContainerTyckInfo { type_id, params: _ }) = tyck_info {
            TypeId::of::<TestContainer<()>>() == *type_id
        } else {
            false
        }
    }

    fn type_name() -> String {
        "TestContainer".into()
    }

    fn children(vself: *const GenericTestContainer) -> ChildrenType {
        let vself: &GenericTestContainer = unsafe { &*vself };
        let iter: Box<dyn Iterator<Item=Value> + '_> =
            Box::new(vself.elements.iter().map(|x: &Value| *x));
        let iter: Box<dyn Iterator<Item=Value> + 'static> = unsafe { transmute::<>(iter) };
        Some(iter)
    }
}

#[repr(transparent)]
pub struct TestContainer<T: 'static> {
    pub inner: GenericTestContainer,
    _phantom: PhantomData<T>
}

impl<T: 'static> TestContainer<T>
    where Void: StaticBase<T>
{
    pub fn new() -> Self {
        Self {
            inner: GenericTestContainer::new(),
            _phantom: PhantomData
        }
    }
}

impl<T: 'static> StaticBase<TestContainer<T>> for Void
    where Void: StaticBase<T>
{
    fn type_id() -> TypeId {
        <Void as StaticBase<GenericTestContainer>>::type_id()
    }

    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        let elem_tyck_info: NonNull<TyckInfo> = <Void as StaticBase<T>>::tyck_info(tyck_info_pool);
        tyck_info_pool.create_container_type(
            <Void as StaticBase<GenericTestContainer>>::type_id(),
            &[elem_tyck_info]
        )
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(ContainerTyckInfo { type_id, params }) = tyck_info {
            let params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
            <Void as StaticBase<GenericTestContainer>>::type_id() == *type_id
                && params.len() == 1
                && <Void as StaticBase<T>>::tyck(unsafe { params.get_unchecked(0).as_ref() })
        } else {
            false
        }
    }

    fn type_name() -> String {
        "TestContainer".into()
    }

    fn children(vself: *const TestContainer<T>) -> ChildrenType {
        <Void as StaticBase<GenericTestContainer>>::children(vself as *const GenericTestContainer)
    }
}

pub fn create_test_container_vt<T: 'static>(tyck_info_pool: &mut TyckInfoPool) -> GenericTypeVT
    where Void: StaticBase<T>
{
    let elem_tyck_info: NonNull<TyckInfo> = <Void as StaticBase<T>>::tyck_info(tyck_info_pool);
    let tyck_info: NonNull<TyckInfo> =
        tyck_info_pool.create_container_type(TypeId::of::<TestContainer<()>>(), &[elem_tyck_info]);

    use crate::data::generic::gen_impls;
    GenericTypeVT {
        tyck_info: unsafe { tyck_info.as_ref().get_container_tyck_info_unchecked() },
        type_name: "TestContainer".to_string(),
        #[cfg(debug_assertions)]
        move_out_fn: gen_impls::generic_move_out_ck::<GenericTestContainer>,
        #[cfg(not(debug_assertions))]
        move_out_fn: gen_impls::generic_move_out::<GenericTestContainer>,
        children_fn: gen_impls::generic_children::<GenericTestContainer>,
        drop_fn: gen_impls::generic_drop::<GenericTestContainer>
    }
}
