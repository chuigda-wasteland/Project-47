use std::any::TypeId;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

use xjbutil::mem::move_to_heap;
use xjbutil::unchecked::UncheckedCellOps;
use xjbutil::void::Void;

use crate::data::generic::GenericTypeVT;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::data::Value;
use crate::data::wrapper::Wrapper;

#[repr(transparent)]
pub struct VMGenericVec {
    pub inner: UnsafeCell<Vec<Value>>
}

impl StaticBase<VMGenericVec> for Void {
    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        tyck_info_pool.create_container_type(
            TypeId::of::<VMGenericVec>(),
            &[tyck_info_pool.get_any_type()]
        )
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(container_tyck_info) = tyck_info {
            if container_tyck_info.type_id != TypeId::of::<VMGenericVec>() {
                return false;
            }

            unsafe {
                container_tyck_info.params.as_ref()[0].as_ref().is_any()
            }
        } else {
            false
        }
    }

    fn children(vself: *const VMGenericVec) -> ChildrenType {
        unsafe {
            let iter = Box::new((*vself).inner.get_ref_unchecked()
                .iter()
                .map(|value: &Value| *value));
            Some(iter)
        }
    }
}

#[repr(transparent)]
pub struct VMVec<T: 'static> {
    pub inner: VMGenericVec,
    _phantom: PhantomData<T>
}

impl<T> StaticBase<VMVec<T>> for Void
    where T: 'static,
          Void: StaticBase<T>
{
    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        tyck_info_pool.create_container_type(
            TypeId::of::<VMVec<T>>(),
            &[tyck_info_pool.get_any_type()]
        )
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Container(container_tyck_info) = tyck_info {
            if container_tyck_info.type_id != TypeId::of::<VMGenericVec>() {
                return false;
            }

            unsafe {
                let child_tyck_info: &TyckInfo = container_tyck_info.params.as_ref()[0].as_ref();
                !child_tyck_info.is_any() && <Void as StaticBase<T>>::tyck(child_tyck_info)
            }
        } else {
            false
        }
    }

    fn children(vself: *const VMVec<T>) -> ChildrenType {
        unsafe {
            let iter = Box::new((*vself).inner.inner.get_ref_unchecked()
                .iter()
                .map(|value: &Value| *value));
            Some(iter)
        }
    }
}

pub struct VMVecRef<T: 'static> {
    pub ptr: *mut Wrapper<()>,
    _phantom: PhantomData<T>
}

pub fn create_vm_vec_vt(
    tyck_info_pool: &mut TyckInfoPool,
    arg_type: NonNull<TyckInfo>
) -> GenericTypeVT {
    let tyck_info: NonNull<TyckInfo> =
        tyck_info_pool.create_container_type(TypeId::of::<VMGenericVec>(), &[arg_type]);

    use crate::data::generic::gen_impls;
    GenericTypeVT {
        tyck_info: unsafe { tyck_info.as_ref().get_container_tyck_info_unchecked() },
        type_name: "vector".to_string(),
        #[cfg(debug_assertions)]
        move_out_fn: gen_impls::generic_move_out_ck::<VMGenericVec>,
        #[cfg(not(debug_assertions))]
        move_out_fn: gen_impls::generic_move_out::<VMGenericVec>,
        children_fn: gen_impls::generic_children::<VMGenericVec>,
        drop_fn: gen_impls::generic_drop::<VMGenericVec>
    }
}

pub fn vec_ctor() -> *mut Wrapper<()> {
    move_to_heap(Wrapper::new_owned(VMGenericVec {
        inner: UnsafeCell::new(Vec::new())
    })).as_ptr() as *mut _
}
