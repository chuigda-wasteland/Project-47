use std::marker::PhantomData;

use crate::data::Value;
use crate::data::wrapper::Wrapper;

#[repr(C)]
pub struct VMGenericVec {
    inner: Vec<Value>
}

#[repr(transparent)]
pub struct VMVec<T: 'static> {
    inner: VMGenericVec,
    _phantom: PhantomData<T>
}

pub struct VMVecRef<T: 'static> {
    ptr: *mut Wrapper<()>
}
