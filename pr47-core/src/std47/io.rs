use std::any::TypeId;
use unchecked_unwrap::UncheckedUnwrap;

use xjbutil::unchecked::UnsafeFrom;

use crate::data::exception::UncheckedException;
use crate::data::tyck::TyckInfoPool;
use crate::data::Value;
use crate::data::value_typed::ValueTypeTag;
use crate::data::wrapper::OWN_INFO_READ_MASK;
use crate::ffi::{FFIException, Signature};
use crate::ffi::sync_fn::{FunctionBase, VMContext};

pub struct PrintBind();

impl FunctionBase for PrintBind {
    fn signature(_tyck_info_pool: &mut TyckInfoPool) -> Signature {
        unimplemented!("print does not have a standard signature")
    }

    unsafe fn call_rtlc<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        for arg in args {
            if arg.is_value() {
                match ValueTypeTag::unsafe_from(arg.vt_data.tag as u8) {
                    ValueTypeTag::Int => print!("{}", arg.vt_data.inner.int_value),
                    ValueTypeTag::Float => print!("{}", arg.vt_data.inner.float_value),
                    ValueTypeTag::Char => print!("{}", arg.vt_data.inner.char_value),
                    ValueTypeTag::Bool => print!("{}", arg.vt_data.inner.bool_value)
                }
            } else {
                if !arg.ownership_info().is_readable() {
                    return Err(FFIException::Unchecked(UncheckedException::OwnershipCheckFailure {
                        object: *arg,
                        expected_mask: OWN_INFO_READ_MASK
                    }));
                }
                if !arg.is_container() &&
                    arg.get_as_dyn_base().as_ref().unchecked_unwrap().dyn_type_id()
                        == TypeId::of::<String>() {
                    print!("{}", &*(arg.get_as_mut_ptr_norm::<String>() as *const _));
                } else {
                    print!("[object Object]");
                }
            }
        }
        Ok(())
    }

    unsafe fn call_unchecked<CTX: VMContext>(
        _context: &mut CTX,
        args: &[Value],
        _rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        for arg in args {
            if arg.is_value() {
                match ValueTypeTag::unsafe_from(arg.vt_data.tag as u8) {
                    ValueTypeTag::Int => print!("{}", arg.vt_data.inner.int_value),
                    ValueTypeTag::Float => print!("{}", arg.vt_data.inner.float_value),
                    ValueTypeTag::Char => print!("{}", arg.vt_data.inner.char_value),
                    ValueTypeTag::Bool => print!("{}", arg.vt_data.inner.bool_value)
                }
            } else if !arg.is_container() &&
                arg.get_as_dyn_base().as_ref().unchecked_unwrap().dyn_type_id()
                    == TypeId::of::<String>() {
                print!("{}", &*(arg.get_as_mut_ptr_norm::<String>() as *const _));
            } else {
                print!("[object Object]");
            }
        }
        Ok(())
    }
}

pub const PRINT_BIND: &PrintBind = &PrintBind();
