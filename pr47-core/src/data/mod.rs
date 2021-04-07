pub mod custom_vt;
pub mod traits;
pub mod tyck;
pub mod value_typed;
pub mod wrapper;

use crate::util::mem::FatPointer;
use crate::data::wrapper::DynBase;
use crate::data::value_typed::ValueTypedData;

#[repr(C)]
#[derive(Clone, Copy)]
pub union Value {
    pub ptr: *mut dyn DynBase,
    pub ptr_repr: FatPointer,
    pub vt_data: ValueTypedData,
}
