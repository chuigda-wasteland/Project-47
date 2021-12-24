use std::any::TypeId;
use std::iter::Iterator;
use std::ptr::NonNull;

use xjbutil::void::Void;

use crate::data::Value;
use crate::data::tyck::{TyckInfo, TyckInfoPool};

pub type ChildrenType = Option<Box<dyn Iterator<Item=Value> + 'static>>;

pub trait StaticBase<T: 'static> {
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        tyck_info_pool.create_plain_type(TypeId::of::<T>())
    }

    fn tyck(tyck_info: &TyckInfo) -> bool {
        if let TyckInfo::Plain(type_id) = tyck_info {
            TypeId::of::<T>() == *type_id
        } else {
            false
        }
    }

    fn type_name() -> String {
        std::any::type_name::<T>().into()
    }

    #[inline] fn children(_vself: *const T) -> ChildrenType { None }
}

// impl !StaticBase<i64> for Void {}
// impl !StaticBase<f64> for Void {}
// impl !StaticBase<char> for Void {}
// impl !StaticBase<bool> for Void {}
// impl<T> !StaticBase<Option<T>> for Void {}
// impl<T, E> !StaticBase<Result<T>> for Void {}

impl StaticBase<String> for Void {
    fn type_name() -> String {
        "string".into()
    }
}

pub trait VMType<T: 'static> {}

impl<T> VMType<T> for Void where T: 'static, Void: StaticBase<T> {}
impl VMType<i64> for Void {}
impl VMType<f64> for Void {}
impl VMType<char> for Void {}
impl VMType<bool> for Void {}
