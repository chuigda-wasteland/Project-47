use std::any::TypeId;
use std::ptr::NonNull;

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::unchecked::UnsafeFrom;
use crate::data::generic::GenericTypeVT;

use crate::data::tyck::TyckInfo;
use crate::data::Value;
use crate::data::value_typed::ValueTypeTag;

#[inline(never)]
pub unsafe fn check_type(value: Value, tyck_info: NonNull<TyckInfo>) -> bool {
    match tyck_info.as_ref() {
        TyckInfo::AnyType => true,
        TyckInfo::Plain(plain) => if value.is_value() {
            match ValueTypeTag::unsafe_from(value.vt_data.tag as u8) {
                ValueTypeTag::Int => *plain == TypeId::of::<i64>(),
                ValueTypeTag::Float => *plain == TypeId::of::<f64>(),
                ValueTypeTag::Char => *plain == TypeId::of::<char>(),
                ValueTypeTag::Bool => *plain == TypeId::of::<bool>(),
            }
        } else if !value.is_container() {
            value.get_as_dyn_base().as_ref().unchecked_unwrap().dyn_tyck(tyck_info.as_ref())
        } else {
            false
        },
        TyckInfo::Nullable(inner) => {
            if value.is_null() {
                true
            } else {
                check_type(value, *inner)
            }
        }
        TyckInfo::Container(_) => {
            if value.is_container() {
                todo!()
            } else if value.is_ref() {
                value.get_as_dyn_base().as_ref().unchecked_unwrap().dyn_tyck(tyck_info.as_ref())
            } else {
                false
            }
        }
        TyckInfo::Function(_) => {
            if value.is_container() {
                let _vt: *const GenericTypeVT = value.ptr_repr.trivia as _;
                todo!()
            } else {
                false
            }
        }
    }
}
