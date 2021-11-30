use std::cell::UnsafeCell;
use std::marker::PhantomData;

use xjbutil::void::Void;

use crate::data::traits::StaticBase;
use crate::data::Value;
use crate::data::wrapper::Wrapper;

#[repr(transparent)]
pub struct VMGenericVec {
    pub inner: UnsafeCell<Vec<Value>>
}

impl StaticBase<VMGenericVec> for Void {}

#[repr(transparent)]
pub struct VMVec<T: 'static> {
    pub inner: VMGenericVec,
    _phantom: PhantomData<T>
}

pub struct VMVecRef<T: 'static> {
    pub ptr: *mut Wrapper<()>,
    _phantom: PhantomData<T>
}
