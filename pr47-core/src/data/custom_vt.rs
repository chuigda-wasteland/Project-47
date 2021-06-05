use std::any::TypeId;
use std::ptr::NonNull;

use crate::data::tyck::ContainerTyckInfo;

pub const CONTAINER_MASK: u8 = 0b00000_010;

#[cfg(debug_assertions)]
pub type MoveOutCkFn = fn(this: *mut (), out: *mut (), type_id: TypeId);
#[cfg(not(debug_assertions))]
pub type MoveOutFn = fn(this: *mut (), out: *mut ());

pub struct ContainerVT {
    pub tyck_info: NonNull<ContainerTyckInfo>,
    pub type_name: String,
    #[cfg(debug_assertions)]
    pub move_out_fn: MoveOutCkFn,
    #[cfg(not(debug_assertions))]
    pub move_out_fn: MoveOutFn
}

impl ContainerVT {
    #[cfg(debug_assertions)]
    pub fn new(
        tyck_info: NonNull<ContainerTyckInfo>,
        type_name: impl ToString,
        move_out_fn: MoveOutCkFn
    ) -> Self {
        Self {
            tyck_info,
            type_name: type_name.to_string(),
            move_out_fn
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(
        tyck_info: NonNull<ContainerTyckInfo>,
        type_name: impl ToString,
        move_out_fn: MoveOutFn
    ) -> Self {
        Self {
            tyck_info,
            type_name: type_name.to_string(),
            move_out_fn
        }
    }
}

#[derive(Clone, Copy)]
pub struct ContainerPtr {
    pub data_ptr: *mut u8,
    pub vt: *mut ContainerVT
}
