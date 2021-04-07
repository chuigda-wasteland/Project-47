use std::ptr::NonNull;
use crate::data::tyck::ContainerTyckInfo;

pub const CONTAINER_MASK: u8 = 0b00000_010;

pub struct ContainerVT {
    pub tyck_info: NonNull<ContainerTyckInfo>,
    pub type_name: String
}

#[derive(Clone, Copy)]
pub struct ContainerPtr {
    pub data_ptr: *mut u8,
    pub vt: *mut ContainerVT
}
