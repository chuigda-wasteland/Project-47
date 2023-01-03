use std::any::TypeId;
use std::ptr::NonNull;

use xjbutil::unchecked::UnsafeFrom;

use crate::data::generic::GenericTypeVT;
use crate::data::tyck::TyckInfo;
use crate::data::Value;
use crate::data::value_typed::{VALUE_TYPE_TAG_MASK, ValueTypeTag};

#[inline(never)]
pub unsafe fn check_type(value: Value, tyck_info: NonNull<TyckInfo>) -> bool {
    match tyck_info.as_ref() {
        TyckInfo::AnyType => true,
        TyckInfo::Plain(plain) => if value.is_value() {
            match ValueTypeTag::unsafe_from((value.vt_data.tag as u8) & VALUE_TYPE_TAG_MASK) {
                ValueTypeTag::Int => *plain == TypeId::of::<i64>(),
                ValueTypeTag::Float => *plain == TypeId::of::<f64>(),
                ValueTypeTag::Char => *plain == TypeId::of::<char>(),
                ValueTypeTag::Bool => *plain == TypeId::of::<bool>(),
            }
        } else if !value.is_container() {
            value.get_as_dyn_base().as_ref().unwrap_unchecked().dyn_tyck(tyck_info.as_ref())
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
        TyckInfo::Container(inner) => {
            if value.is_container() {
                let vt: *const GenericTypeVT = value.ptr_repr.trivia as *const GenericTypeVT;
                let vt: &GenericTypeVT = &*vt;

                if vt.tyck_info.as_ref().type_id != inner.type_id {
                    return false;
                }

                // TODO this is temporary patch for LY testing
                vt.tyck_info.as_ref().params.as_ref().len() == inner.params.as_ref().len()
            } else if value.is_ref() {
                value.get_as_dyn_base().as_ref().unwrap_unchecked().dyn_tyck(tyck_info.as_ref())
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
