use std::any::TypeId;
use std::collections::HashSet;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::mem::transmute;
use std::ptr::NonNull;

pub struct ContainerTyckInfo {
    pub type_id: TypeId,
    pub params: NonNull<[NonNull<TyckInfo>]>
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
                let params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
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
                let self_params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
                if let TyckInfo::Container(other_container_tyck_info) = other {
                    let other_params: &[NonNull<TyckInfo>] = unsafe {
                        other_container_tyck_info.params.as_ref()
                    };
                    *type_id == other_container_tyck_info.type_id
                    && self_params.iter().zip(other_params.iter()).all(
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
    pool: HashSet<Box<TyckInfo>>
}

impl TyckInfoPool {
    pub fn new() -> Self {
        let mut pool: HashSet<Box<TyckInfo>> = HashSet::new();
        pool.insert(Box::new(TyckInfo::Plain(TypeId::of::<String>())));

        Self { pool }
    }

    pub fn get_plain_type<'a>(&'a mut self, type_id: TypeId) -> &'a TyckInfo {
        let tyck_info: TyckInfo = TyckInfo::Plain(type_id);
        if let Some(tyck_info /*: &Box<TyckInfo>*/) = self.pool.get(&tyck_info) {
            let ret: &'_ TyckInfo = tyck_info.as_ref();
            unsafe { transmute::<_, &'a TyckInfo>(ret) }
        } else {
            let tyck_info: Box<TyckInfo> = Box::new(tyck_info);
            let ret: &'_ TyckInfo = tyck_info.as_ref();
            let ret: &'a TyckInfo = unsafe { transmute::<_, &'a TyckInfo>(ret) };
            self.pool.insert(tyck_info);
            ret
        }
    }
}
