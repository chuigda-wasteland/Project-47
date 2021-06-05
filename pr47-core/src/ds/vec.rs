use crate::data::Value;

#[repr(C)]
pub struct VMGenericVec {
    inner: Vec<Value>
}
