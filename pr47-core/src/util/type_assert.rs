use crate::util::void::Void;

pub trait AssertClone<T> {}
pub trait AssertRef<T> {}
pub trait AssertMutRef<T> {}
pub trait AssertResult<T> {}
pub trait AssertOption<T> {}

impl<T: Clone> AssertClone<T> for Void {}

impl<T> AssertRef<&T> for Void {}
impl<T> AssertRef<&mut T> for Void {}
impl<T> AssertMutRef<&mut T> for Void {}

impl<T, E> AssertResult<core::result::Result<T, E>> for Void {}
impl<T> AssertOption<core::option::Option<T>> for Void {}
