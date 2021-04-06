use std::any::TypeId;

pub enum TyckInfo {
    SimpleType(TypeId),
    ContainerType(TypeId, *const Vec<TyckInfo>)
}
