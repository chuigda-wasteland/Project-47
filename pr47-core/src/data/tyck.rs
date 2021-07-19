use std::any::TypeId;
use std::collections::HashSet;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;

pub struct ContainerTyckInfo {
    pub type_id: TypeId,
    pub params: Vec<NonNull<TyckInfo>>
}

pub enum TyckInfo {
    Plain(TypeId),
    Container(ContainerTyckInfo)
}

impl Hash for TyckInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TyckInfo::Plain(plain_type_id) => {
                plain_type_id.hash(state);
            },
            TyckInfo::Container(ContainerTyckInfo { type_id, params }) => {
                type_id.hash(state);
                for param /*: &NonNull<TyckInfo>*/ in params.iter() {
                    state.write_usize(param.as_ptr() as usize);
                }
            }
        }
    }
}

impl PartialEq for TyckInfo {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TyckInfo::Plain(plain_type_id) => {
                if let TyckInfo::Plain(other_plain_type_id) = other {
                    plain_type_id == other_plain_type_id
                } else {
                    false
                }
            },
            TyckInfo::Container(ContainerTyckInfo { type_id, params }) => {
                if let TyckInfo::Container(other_container_tyck_info) = other {
                    *type_id == other_container_tyck_info.type_id
                    && params.iter().zip(other_container_tyck_info.params.iter()).all(
                        |(p1, p2): (&NonNull<TyckInfo>, &NonNull<TyckInfo>)| {
                            p1.as_ptr() == p2.as_ptr()
                        }
                    )
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for TyckInfo {}

pub struct TyckInfoPool {
    pool: HashSet<NonNull<TyckInfo>>
}

impl Drop for TyckInfoPool {
    fn drop(&mut self) {
        for tyck_info /*: &NonNull<TyckInfo>*/ in self.pool.iter() {
            let tyck_info: Box<TyckInfo> = unsafe { Box::from_raw(tyck_info.as_ptr()) };
            drop(tyck_info);
        }
    }
}

impl TyckInfoPool {
    pub fn new() -> Self {
        let mut pool: HashSet<NonNull<TyckInfo>> = HashSet::new();
        let string_type: Box<TyckInfo> = Box::new(TyckInfo::Plain(TypeId::of::<String>()));
        let string_type: NonNull<TyckInfo> = unsafe {
            NonNull::new_unchecked(Box::leak(string_type) as *mut _)
        };
        pool.insert(string_type);

        Self { pool }
    }
}
