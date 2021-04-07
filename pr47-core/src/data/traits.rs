use std::any::TypeId;
use crate::data::tyck::TyckInfo;
use crate::util::void::Void;

pub trait StaticBase<T: 'static> {
    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn tyck_info() -> TyckInfo {
        TyckInfo::Plain(TypeId::of::<T>())
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
