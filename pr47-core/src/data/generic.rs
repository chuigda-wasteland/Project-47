use std::fmt::{Debug, Formatter};
use std::ptr::NonNull;

use crate::data::traits::ChildrenType;
use crate::data::tyck::ContainerTyckInfo;
use crate::data::wrapper::Wrapper;

#[cfg(debug_assertions)] use std::any::TypeId;

pub trait GenericTypeRef {
    unsafe fn create_ref(wrapper_ptr: *mut Wrapper<()>) -> Self;
}

#[allow(clippy::unusual_byte_groupings)]
pub const GENERIC_TYPE_MASK: u8 = 0b00000_010;

#[cfg(debug_assertions)]
pub type MoveOutCkFn = unsafe fn(this: *mut (), out: *mut (), type_id: TypeId);
#[cfg(not(debug_assertions))]
pub type MoveOutFn = unsafe fn(this: *mut (), out: *mut ());

pub type ChildrenFn = unsafe fn(this: *const ()) -> ChildrenType;

pub type DropFn = unsafe fn(this: *mut());

pub type GenericTypeCtor = fn() -> *mut Wrapper<()>;

pub struct GenericTypeVT {
    pub tyck_info: NonNull<ContainerTyckInfo>,
    pub type_name: String,
    #[cfg(debug_assertions)]
    pub move_out_fn: MoveOutCkFn,
    #[cfg(not(debug_assertions))]
    pub move_out_fn: MoveOutFn,
    pub children_fn: ChildrenFn,
    pub drop_fn: DropFn
}

impl GenericTypeVT {
    #[cfg(debug_assertions)]
    pub fn new(
        tyck_info: NonNull<ContainerTyckInfo>,
        type_name: impl ToString,
        move_out_fn: MoveOutCkFn,
        children_fn: ChildrenFn,
        drop_fn: DropFn
    ) -> Self {
        Self {
            tyck_info,
            type_name: type_name.to_string(),
            move_out_fn,
            children_fn,
            drop_fn
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(
        tyck_info: NonNull<ContainerTyckInfo>,
        type_name: impl ToString,
        move_out_fn: MoveOutFn,
        children_fn: ChildrenFn,
        drop_fn: DropFn
    ) -> Self {
        Self {
            tyck_info,
            type_name: type_name.to_string(),
            move_out_fn,
            children_fn,
            drop_fn
        }
    }
}

impl Debug for GenericTypeVT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContainerVT({})", self.type_name)
    }
}

#[derive(Clone, Copy)]
pub struct ContainerPtr {
    pub data_ptr: *mut u8,
    pub vt: *mut GenericTypeVT
}

pub mod gen_impls {
    use std::mem::{MaybeUninit, ManuallyDrop};

    use xjbutil::void::Void;

    use crate::data::traits::{ChildrenType, StaticBase};
    use crate::data::wrapper::{OwnershipInfo, Wrapper};

    #[cfg(debug_assertions)] use std::any::TypeId;

    #[cfg(debug_assertions)]
    #[inline(always)]
    pub unsafe fn generic_move_out_ck<T>(this: *mut (), out: *mut (), type_id: TypeId)
        where T: 'static,
              Void: StaticBase<T>
    {
        assert_eq!(type_id, TypeId::of::<T>());
        let this: &mut Wrapper<T> = &mut *(this as *mut Wrapper<_>);
        let out: &mut MaybeUninit<T> = &mut *(out as *mut MaybeUninit<_>);

        assert_eq!(this.ownership_info, OwnershipInfo::VMOwned as u8);
        let data: T = ManuallyDrop::take(&mut this.data.owned).assume_init();
        std::ptr::write(out.as_mut_ptr(), data);
        this.ownership_info = OwnershipInfo::MovedToRust as u8;
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub unsafe fn generic_move_out<T>(this: *mut (), out: *mut ())
        where T: 'static,
              Void: StaticBase<T>
    {
        let this: &mut Wrapper<T> = &mut *(this as *mut Wrapper<_>);
        let out: &mut MaybeUninit<T> = &mut *(out as *mut MaybeUninit<_>);

        let data: T = ManuallyDrop::take(&mut this.data.owned).assume_init();
        std::ptr::write(out.as_mut_ptr(), data);
        this.ownership_info = OwnershipInfo::MovedToRust as u8;
    }

    #[inline(always)]
    pub unsafe fn generic_children<T>(this: *const ()) -> ChildrenType
        where T: 'static,
              Void: StaticBase<T>
    {
        <Void as StaticBase<T>>::children(this as *const _)
    }

    #[inline(always)]
    pub unsafe fn generic_drop<T>(this: *mut ())
        where T: 'static,
              Void: StaticBase<T>
    {
        let boxed: Box<Wrapper<T>> = Box::from_raw(this as *mut _);
        drop(boxed);
    }
}
