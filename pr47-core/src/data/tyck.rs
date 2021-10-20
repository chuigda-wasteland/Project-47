use std::any::TypeId;
use std::collections::HashSet;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::hint::unreachable_unchecked;
use std::mem::forget;
use std::ptr::NonNull;

use xjbutil::korobka::Korobka;
use xjbutil::std_ext::{BoxedExt, VecExt};

pub struct ContainerTyckInfo {
    pub type_id: TypeId,
    pub params: NonNull<[NonNull<TyckInfo>]>
}

pub enum TyckInfo {
    Plain(TypeId),
    Container(ContainerTyckInfo)
}

impl TyckInfo {
    pub unsafe fn get_container_tyck_info_unchecked(&self) -> NonNull<ContainerTyckInfo> {
        if let TyckInfo::Container(container_tyck_info) = self {
            NonNull::new_unchecked(container_tyck_info as *const _ as *mut _)
        } else {
            unreachable_unchecked()
        }
    }
}

impl Drop for TyckInfo {
    fn drop(&mut self) {
        if let TyckInfo::Container(ContainerTyckInfo { type_id, params }) = self {
            let _ = type_id;
            let boxed: Box<[NonNull<TyckInfo>]> = unsafe { Box::reclaim(*params) };
            drop(boxed);
        }
    }
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
    pool: HashSet<Korobka<TyckInfo>>
}

impl TyckInfoPool {
    pub fn new() -> Self {
        let mut pool: HashSet<Korobka<TyckInfo>> = HashSet::new();
        pool.insert(Korobka::new(TyckInfo::Plain(TypeId::of::<String>())));

        Self { pool }
    }

    pub fn create_plain_type(&mut self, type_id: TypeId) -> NonNull<TyckInfo> {
        let tyck_info: TyckInfo = TyckInfo::Plain(type_id);
        if let Some(tyck_info /*: &Korobka<TyckInfo>*/) = self.pool.get(&tyck_info) {
            tyck_info.as_nonnull()
        } else {
            let tyck_info: Korobka<TyckInfo> = Korobka::new(tyck_info);
            let ret: NonNull<TyckInfo> = tyck_info.as_nonnull();
            self.pool.insert(tyck_info);
            ret
        }
    }

    pub fn create_container_type(
        &mut self,
        container_type_id: TypeId,
        params: &[NonNull<TyckInfo>]
    ) -> NonNull<TyckInfo> {
        let query_tyck_info: TyckInfo = TyckInfo::Container(ContainerTyckInfo {
            type_id: container_type_id,
            params: unsafe { NonNull::new_unchecked(params as *const _ as *mut _) }
        });

        let ret: NonNull<TyckInfo> = 
            if let Some(tyck_info /*: &Korobka<TyckInfo>*/) = self.pool.get(&query_tyck_info) {
                tyck_info.as_nonnull()
            } else {
                let tyck_info: TyckInfo = TyckInfo::Container(ContainerTyckInfo {
                    type_id: container_type_id,
                    params: Vec::from(params).into_slice_ptr()
                });
                let tyck_info: Korobka<TyckInfo> = Korobka::new(tyck_info);
                let ret: NonNull<TyckInfo> = tyck_info.as_nonnull();
                self.pool.insert(tyck_info);
                ret
            };

        forget(query_tyck_info);
        ret
    }
}

#[cfg(test)]
mod test_tyck_info_pool {
    use std::any::TypeId;
    use std::ptr::NonNull;

    use crate::data::tyck::{TyckInfo, TyckInfoPool};

    struct TestType1();
    struct TestType2();
    struct TestType3();

    #[test]
    fn test_tyck_info_pool() {
        let type1_id: TypeId = TypeId::of::<TestType1>();
        let type2_id: TypeId = TypeId::of::<TestType2>();
        let type3_id: TypeId = TypeId::of::<TestType3>();

        let mut tyck_info_pool: TyckInfoPool = TyckInfoPool::new();

        let tyck_info1: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(type1_id);
        let tyck_info2: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(type2_id);
        let tyck_info3: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(type3_id);

        let tyck_info1_1: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(type1_id);
        let tyck_info2_1: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(type2_id);
        let tyck_info3_1: NonNull<TyckInfo> = tyck_info_pool.create_plain_type(type3_id);

        assert_eq!(tyck_info1, tyck_info1_1);
        assert_eq!(tyck_info2, tyck_info2_1);
        assert_eq!(tyck_info3, tyck_info3_1);

        assert_ne!(tyck_info1, tyck_info2);
        assert_ne!(tyck_info1, tyck_info3);
        assert_ne!(tyck_info2, tyck_info3);

        let type4_params: [NonNull<TyckInfo>; 2] = [tyck_info1, tyck_info2];
        let type5_params: [NonNull<TyckInfo>; 2] = [tyck_info2, tyck_info3];
        let type6_params: [NonNull<TyckInfo>; 2] = [tyck_info1, tyck_info3];

        let tyck_info4: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type3_id, &type4_params);
        let tyck_info5: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type3_id, &type5_params);
        let tyck_info6: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type3_id, &type6_params);

        let tyck_info4_1: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type3_id, &type4_params);
        let tyck_info5_1: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type3_id, &type5_params);
        let tyck_info6_1: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type3_id, &type6_params);

        assert_eq!(tyck_info4, tyck_info4_1);
        assert_eq!(tyck_info5, tyck_info5_1);
        assert_eq!(tyck_info6, tyck_info6_1);

        assert_ne!(tyck_info4, tyck_info5);
        assert_ne!(tyck_info5, tyck_info6);
        assert_ne!(tyck_info4, tyck_info6);

        let type7_params: [NonNull<TyckInfo>; 3] = [tyck_info4, tyck_info5, tyck_info6];
        let type9_params: [NonNull<TyckInfo>; 3] = [tyck_info6, tyck_info4, tyck_info5];

        let tyck_info7: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type1_id, &type7_params);
        let tyck_info8: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type2_id, &type7_params);
        let tyck_info9: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type1_id, &type9_params);

        let tyck_info7_1: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type1_id, &type7_params);
        let tyck_info8_1: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type2_id, &type7_params);
        let tyck_info9_1: NonNull<TyckInfo> =
            tyck_info_pool.create_container_type(type1_id, &type9_params);

        assert_eq!(tyck_info7, tyck_info7_1);
        assert_eq!(tyck_info8, tyck_info8_1);
        assert_eq!(tyck_info9, tyck_info9_1);

        assert_ne!(tyck_info7, tyck_info8);
        assert_ne!(tyck_info7, tyck_info9);
        assert_ne!(tyck_info8, tyck_info9);
    }
}
