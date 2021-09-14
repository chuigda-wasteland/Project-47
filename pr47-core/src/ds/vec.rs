use std::marker::PhantomData;

use crate::data::Value;
use crate::data::wrapper::Wrapper;

#[repr(C)]
pub struct VMGenericVec {
    pub inner: Vec<Value>
}

#[repr(transparent)]
pub struct VMVec<T: 'static> {
    pub inner: VMGenericVec,
    _phantom: PhantomData<T>
}

pub struct VMVecRef<T: 'static> {
    pub ptr: *mut Wrapper<()>,
    _phantom: PhantomData<T>
}
