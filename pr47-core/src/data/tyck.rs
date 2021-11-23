use std::any::TypeId;
use std::collections::HashSet;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::hint::unreachable_unchecked;
use std::mem::{discriminant, forget};
use std::ptr::NonNull;

use xjbutil::korobka::Korobka;
use xjbutil::std_ext::{BoxedExt, VecExt};

use crate::builtins::object::Object;

pub struct ContainerTyckInfo {
    pub type_id: TypeId,
    pub params: NonNull<[NonNull<TyckInfo>]>
}

pub struct FunctionTyckInfo {
    pub params: NonNull<[NonNull<TyckInfo>]>,
    pub rets: NonNull<[NonNull<TyckInfo>]>,
    pub exceptions: NonNull<[NonNull<TyckInfo>]>
}

pub enum TyckInfo {
    AnyType,
    Plain(TypeId),
    Nullable(NonNull<TyckInfo>),
    Container(ContainerTyckInfo),
    Function(FunctionTyckInfo)
}

impl TyckInfo {
    pub unsafe fn get_container_tyck_info_unchecked(&self) -> NonNull<ContainerTyckInfo> {
        if let TyckInfo::Container(container_tyck_info) = self {
            NonNull::new_unchecked(container_tyck_info as *const _ as *mut _)
        } else {
            unreachable_unchecked()
        }
    }

    pub unsafe fn get_function_tyck_info_unchecked(&self) -> NonNull<FunctionTyckInfo> {
        if let TyckInfo::Function(function_tyck_info) = self {
            NonNull::new_unchecked(function_tyck_info as *const _ as *mut _)
        } else {
            unreachable_unchecked()
        }
    }
}

impl Drop for TyckInfo {
    fn drop(&mut self) {
        match self {
            TyckInfo::Container(ContainerTyckInfo { params, .. }) => {
                let boxed: Box<[NonNull<TyckInfo>]> = unsafe { Box::reclaim(*params) };
                drop(boxed);
            },
            TyckInfo::Function(FunctionTyckInfo { params, rets, exceptions }) => {
                let boxed: Box<[NonNull<TyckInfo>]> = unsafe { Box::reclaim(*params) };
                drop(boxed);
                let boxed: Box<[NonNull<TyckInfo>]> = unsafe { Box::reclaim(*rets) };
                drop(boxed);
                let boxed: Box<[NonNull<TyckInfo>]> = unsafe { Box::reclaim(*exceptions) };
                drop(boxed);
            },
            _ => {}
        }
    }
}

impl Hash for TyckInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TyckInfo::AnyType => {
                discriminant(self).hash(state);
            },
            TyckInfo::Plain(plain_type_id) => {
                discriminant(self).hash(state);
                plain_type_id.hash(state);
            },
            TyckInfo::Nullable(underlying) => {
                discriminant(self).hash(state);
                underlying.as_ptr().hash(state);
            },
            TyckInfo::Container(ContainerTyckInfo { type_id, params }) => {
                discriminant(self).hash(state);
                type_id.hash(state);
                let params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
                for param /*: &NonNull<TyckInfo>*/ in params.iter() {
                    state.write_usize(param.as_ptr() as usize);
                }
            },
            TyckInfo::Function(FunctionTyckInfo { params, rets, exceptions }) => {
                discriminant(self).hash(state);
                let params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
                for param /*: &NonNull<TyckInfo>*/ in params.iter() {
                    state.write_usize(param.as_ptr() as usize);
                }

                let rets: &[NonNull<TyckInfo>] = unsafe { rets.as_ref() };
                for ret /*: &NonNull<TyckInfo>*/ in rets.iter() {
                    state.write_usize(ret.as_ptr() as usize);
                }

                let exceptions: &[NonNull<TyckInfo>] = unsafe { exceptions.as_ref() };
                for exception /*: &NonNull<TyckInfo>*/ in exceptions.iter() {
                    state.write_usize(exception.as_ptr() as usize);
                }
            }
        }
    }
}

impl PartialEq for TyckInfo {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TyckInfo::AnyType => {
                if let TyckInfo::AnyType = other { true } else { false }
            },
            TyckInfo::Plain(plain_type_id) => {
                if let TyckInfo::Plain(other_plain_type_id) = other {
                    plain_type_id == other_plain_type_id
                } else {
                    false
                }
            },
            TyckInfo::Nullable(underlying) => {
                if let TyckInfo::Nullable(other_underlying) = other {
                    underlying.as_ptr() == other_underlying.as_ptr()
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
            },
            TyckInfo::Function(FunctionTyckInfo { params, rets, exceptions }) => {
                let self_params: &[NonNull<TyckInfo>] = unsafe { params.as_ref() };
                let self_rets: &[NonNull<TyckInfo>] = unsafe { rets.as_ref() };
                let self_exceptions: &[NonNull<TyckInfo>] = unsafe { exceptions.as_ref() };
                if let TyckInfo::Function(other_function_tyck_info) = other {
                    let other_params: &[NonNull<TyckInfo>] = unsafe {
                        other_function_tyck_info.params.as_ref()
                    };
                    let other_rets: &[NonNull<TyckInfo>] = unsafe {
                        other_function_tyck_info.rets.as_ref()
                    };
                    let other_exceptions: &[NonNull<TyckInfo>] = unsafe {
                        other_function_tyck_info.exceptions.as_ref()
                    };
                    self_params.iter().zip(other_params.iter()).all(
                        |(p1, p2): (&NonNull<TyckInfo>, &NonNull<TyckInfo>)| {
                            p1.as_ptr() == p2.as_ptr()
                        }
                    )
                    && self_rets.iter().zip(other_rets.iter()).all(
                        |(r1, r2): (&NonNull<TyckInfo>, &NonNull<TyckInfo>)| {
                            r1.as_ptr() == r2.as_ptr()
                        }
                    )
                    && self_exceptions.iter().zip(other_exceptions.iter()).all(
                        |(e1, e2): (&NonNull<TyckInfo>, &NonNull<TyckInfo>)| {
                            e1.as_ptr() == e2.as_ptr()
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
    pool: HashSet<Korobka<TyckInfo>>,
    any_type: NonNull<TyckInfo>,
    object_type: NonNull<TyckInfo>,
    string_type: NonNull<TyckInfo>
}

impl TyckInfoPool {
    pub fn new() -> Self {
        let mut pool: HashSet<Korobka<TyckInfo>> = HashSet::new();

        pool.insert(Korobka::new(TyckInfo::AnyType));
        let any_type: NonNull<TyckInfo> = pool.get(&TyckInfo::AnyType).unwrap().as_nonnull();

        let mut ret = Self {
            pool,
            any_type,
            object_type: NonNull::dangling(),
            string_type: NonNull::dangling()
        };

        ret.object_type = ret.create_plain_type(TypeId::of::<Object>());
        ret.string_type = ret.create_plain_type(TypeId::of::<String>());

        ret
    }

    pub fn get_any_type(&self) -> NonNull<TyckInfo> {
        self.any_type
    }

    pub fn get_object_type(&self) -> NonNull<TyckInfo> {
        self.object_type
    }

    pub fn get_string_type(&self) -> NonNull<TyckInfo> {
        self.string_type
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

    pub fn create_nullable_type(&mut self, base: NonNull<TyckInfo>) -> NonNull<TyckInfo> {
        let tyck_info: TyckInfo = TyckInfo::Nullable(base);
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

    pub fn create_function_type(
        &mut self,
        params: &[NonNull<TyckInfo>],
        rets: &[NonNull<TyckInfo>],
        exceptions: &[NonNull<TyckInfo>]
    ) -> NonNull<TyckInfo> {
        let query_tyck_info: TyckInfo = TyckInfo::Function(FunctionTyckInfo {
            params: unsafe { NonNull::new_unchecked(params as *const _ as *mut _) },
            rets: unsafe { NonNull::new_unchecked(rets as *const _ as *mut _) },
            exceptions: unsafe { NonNull::new_unchecked(exceptions as *const _ as *mut _) }
        });

        let ret: NonNull<TyckInfo> =
            if let Some(tyck_info /*: &Korobka<TyckInfo>*/) = self.pool.get(&query_tyck_info) {
                tyck_info.as_nonnull()
            } else {
                let tyck_info: TyckInfo = TyckInfo::Function(FunctionTyckInfo {
                    params: Vec::from(params).into_slice_ptr(),
                    rets: Vec::from(rets).into_slice_ptr(),
                    exceptions: Vec::from(exceptions).into_slice_ptr()
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
