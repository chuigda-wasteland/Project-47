use std::any::TypeId;
use std::ptr::NonNull;

pub enum TyckInfo {
    SimpleType(TypeId),
    ContainerType(TypeId, Vec<NonNull<TyckInfo>>)
}
