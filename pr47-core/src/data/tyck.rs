use std::any::TypeId;
use std::ptr::NonNull;

pub struct ContainerTyckInfo {
    pub type_id: TypeId,
    pub params: Vec<NonNull<TyckInfo>>
}

pub enum TyckInfo {
    Plain(TypeId),
    Container(ContainerTyckInfo)
}
