use std::convert::Infallible;

use xjbutil::wide_ptr::WidePointer;

use crate::data::Value;

#[no_mangle]
pub extern "C" fn pr47_al31fu_rs_rust_panic() -> Infallible {
    panic!()
}

#[no_mangle]
pub unsafe extern "C" fn pr47_al31fu_rs_poll_fut(
    _wide_ptr: WidePointer,
    _ret_values: *mut [*mut Value; 8]
) -> bool {
    todo!()
}
