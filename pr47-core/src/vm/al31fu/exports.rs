use std::alloc::{alloc, dealloc, Layout};

use crate::data::Value;

#[cfg(target_pointer_width = "64")]
use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::wide_ptr::WidePointer;

#[no_mangle]
pub unsafe extern "C" fn pr47_al31fu_export_alloc_values(count: usize) -> *mut u8 {
    #[cfg(target_pointer_width = "32")]
    let layout: Layout = Layout::array::<Value>(count).unwrap();
    #[cfg(target_pointer_width = "64")]
    let layout: Layout = Layout::array::<Value>(count).unchecked_unwrap();

    let ret: *mut u8 = alloc(layout);
    if ret.is_null() {
        panic!();
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn pr47_al31fu_export_dealloc_values(ptr: *mut u8, count: usize) {
    if !ptr.is_null() {
        dealloc(ptr, Layout::array::<Value>(count).unchecked_unwrap());
    }
}

#[no_mangle]
pub unsafe extern "C" fn pr47_al31fu_export_poll_fut(
    _wide_ptr: WidePointer,
    _ret_values: *mut [*mut Value; 8]
) -> bool {
    todo!()
}
