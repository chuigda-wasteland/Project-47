use std::future::Future;
use std::pin::Pin;

use xjbutil::void::Void;

use crate::data::Value;
use crate::data::container::ContainerRef;
use crate::data::exception::UncheckedException;
use crate::data::traits::{StaticBase};
use crate::data::wrapper::{OwnershipInfo, Wrapper};
use crate::data::wrapper::{
    OWN_INFO_READ_MASK,
    OWN_INFO_WRITE_MASK
};
use crate::ffi::{FFIException, Signature};
use crate::util::serializer::Serializer;

pub trait AsyncVMContext: 'static + Sized + Send + Sync {
    type SharedData;

    fn serializer(&self) -> &Serializer<Self::SharedData>;
}

pub trait AsyncFunctionBase: 'static {
    fn signature() -> Signature;

    fn call_tyck<ACTX: AsyncVMContext>(context: &ACTX, args: &[Value])
        -> Result<Promise, FFIException>;

    unsafe fn call_rtlc<ACTX: AsyncVMContext>(context: &ACTX, args: &[Value])
        -> Result<Promise, FFIException>;
}

pub trait AsyncFunction<ACTX: AsyncVMContext>: 'static {
    fn signature(&self) -> Signature;

    fn call_tyck(&self, context: &ACTX, args: &[Value]) -> Result<Promise, FFIException>;

    unsafe fn call_rtlc(&self, context: &ACTX, args: &[Value]) -> Result<Promise, FFIException>;
}

impl<AFBase, CTX> AsyncFunction<CTX> for AFBase where
    AFBase: AsyncFunctionBase,
    CTX: AsyncVMContext
{
    fn signature(&self) -> Signature {
        <AFBase as AsyncFunctionBase>::signature()
    }

    fn call_tyck(&self, context: &CTX, args: &[Value]) -> Result<Promise, FFIException> {
        <AFBase as AsyncFunctionBase>::call_tyck(context, args)
    }

    unsafe fn call_rtlc(&self, context: &CTX, args: &[Value]) -> Result<Promise, FFIException> {
        <AFBase as AsyncFunctionBase>::call_rtlc(context, args)
    }
}

#[derive(Copy, Clone)]
pub struct AsyncResetGuard {
    wrapper_ptr: *mut Wrapper<()>,
    original: u8
}

#[derive(Copy, Clone)]
pub struct AsyncShareGuard {
    wrapper_ptr: *mut Wrapper<()>
}

pub union AsyncOwnershipGuard {
    reset_guard: AsyncResetGuard,
    share_guard: AsyncShareGuard
}

impl AsyncOwnershipGuard {
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn reset(&self) {
        let wrapper_ref: &mut Wrapper<()> = unsafe { &mut *self.reset_guard.wrapper_ptr };
        wrapper_ref.ownership_info = unsafe { self.reset_guard.original }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn un_share(&self) {
        let wrapper_ref: &mut Wrapper<()> = unsafe { &mut *self.share_guard.wrapper_ptr };
        wrapper_ref.refcount -= 1;
        if wrapper_ref.refcount == 0 {
            wrapper_ref.ownership_info = wrapper_ref.ownership_info2;
        }
    }
}

impl From<AsyncResetGuard> for AsyncOwnershipGuard {
    fn from(reset_guard: AsyncResetGuard) -> Self {
        Self { reset_guard }
    }
}

impl From<AsyncShareGuard> for AsyncOwnershipGuard {
    #[inline(always)]
    fn from(share_guard: AsyncShareGuard) -> Self {
        Self { share_guard }
    }
}

pub struct AsyncReturnType(pub Result<Box<[Value]>, FFIException>);

unsafe impl Send for AsyncReturnType {}
unsafe impl Sync for AsyncReturnType {}

pub struct PromiseGuard {
    pub guards: Box<[AsyncOwnershipGuard]>,
    pub reset_guard_count: usize
}

impl Drop for PromiseGuard {
    fn drop(&mut self) {
        let guard_count: usize = self.guards.len();
        for i in 0..self.reset_guard_count {
            unsafe { self.guards.get_unchecked(i).reset() }
        }
        for i in self.reset_guard_count..guard_count {
            unsafe { self.guards.get_unchecked(i).un_share() }
        }
    }
}

pub struct Promise {
    pub fut: Pin<Box<dyn Future<Output=AsyncReturnType> + Send + 'static>>,
    pub guard: PromiseGuard
}

impl Promise {
    pub async fn await_promise(self) -> AsyncReturnType {
        let fut: Pin<Box<dyn Future<Output=AsyncReturnType> + Send + 'static>> = self.fut;
        let result: AsyncReturnType = fut.await;
        result
    }
}

impl StaticBase<Promise> for Void {
    fn type_name() -> String {
        "promise".to_string()
    }
}

pub use crate::ffi::sync_fn::{
    value_copy,
    value_copy_norm,
    value_move_out,
    value_move_out_norm,
    value_move_out_check,
    value_move_out_check_norm
};

#[inline] pub unsafe fn value_into_ref<'a, T>(
    value: Value
) -> Result<(&'a T, AsyncShareGuard), FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        let data_ptr: *const T = value.get_as_mut_ptr_norm() as *const T;
        if original != OwnershipInfo::SharedToRust as u8 {
            (*wrapper_ptr).ownership_info2 = original;
            (*wrapper_ptr).ownership_info = OwnershipInfo::SharedToRust as u8;
            (*wrapper_ptr).refcount = 1;
        } else {
            (*wrapper_ptr).refcount += 1;
        }
        Ok((
            &*data_ptr,
            AsyncShareGuard { wrapper_ptr }
        ))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_ref<CR>(
    value: Value
) -> Result<(CR, AsyncShareGuard), FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        if original != OwnershipInfo::SharedToRust as u8 {
            (*wrapper_ptr).ownership_info2 = original;
            (*wrapper_ptr).ownership_info = OwnershipInfo::SharedToRust as u8;
            (*wrapper_ptr).refcount = 1;
        } else {
            (*wrapper_ptr).refcount -= 1;
        }
        Ok((
            CR::create_ref(wrapper_ptr),
            AsyncShareGuard { wrapper_ptr }
        ))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_mut_ref<'a, T>(
    value: Value
) -> Result<(&'a mut T, AsyncResetGuard), FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_WRITE_MASK != 0 {
        let data_ptr: *mut T = value.get_as_mut_ptr_norm() as *mut T;
        (*wrapper_ptr).ownership_info = OwnershipInfo::MutSharedToRust as u8;
        Ok((
            &mut *data_ptr,
            AsyncResetGuard { wrapper_ptr, original }
        ))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_mut_ref<CR>(
    value: Value
) -> Result<(CR, AsyncResetGuard), FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_WRITE_MASK != 0 {
        (*wrapper_ptr).ownership_info = OwnershipInfo::MutSharedToRust as u8;
        Ok((
            CR::create_ref(wrapper_ptr),
            AsyncResetGuard { wrapper_ptr, original }
        ))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}
