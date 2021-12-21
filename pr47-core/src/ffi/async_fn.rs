use std::future::Future;
use std::pin::Pin;

use xjbutil::void::Void;

use crate::data::Value;
use crate::data::exception::{ExceptionInner, UncheckedException};
use crate::data::generic::GenericTypeRef;
use crate::data::traits::{StaticBase};
use crate::data::wrapper::{OWN_INFO_GLOBAL_MASK, OwnershipInfo, Wrapper};
use crate::data::wrapper::{
    OWN_INFO_OWNED_MASK,
    OWN_INFO_READ_MASK,
    OWN_INFO_WRITE_MASK
};
use crate::ffi::{FFIException, Signature};
use crate::util::serializer::{CoroutineSharedData, Serializer};

pub trait VMDataTrait : 'static + Sized + Send {
    type Alloc;

    fn get_alloc(&mut self) -> &mut Self::Alloc;
}

pub trait AsyncVMContext: 'static + Sized + Send + Sync {
    type VMData: VMDataTrait;

    fn serializer(&self) -> &Serializer<(CoroutineSharedData, Self::VMData)>;
}

pub trait AsyncFunctionBase: 'static {
    fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature;

    unsafe fn call_rtlc<A: Alloc, VD: VMDataTrait<Alloc= A>, ACTX: AsyncVMContext<VMData = VD>> (
        context: &ACTX,
        args: &[Value]
    ) -> Result<Promise<A>, FFIException>;
}

pub trait AsyncFunction<A: Alloc, VD: VMDataTrait, ACTX: AsyncVMContext>: 'static {
    fn signature(&self, tyck_info_pool: &mut TyckInfoPool) -> Signature;

    unsafe fn call_rtlc(&self, context: &ACTX, args: &[Value]) -> Result<Promise<A>, FFIException>;
}

impl<AFBase, A, VD, ACTX> AsyncFunction<A, VD, ACTX> for AFBase where
    AFBase: AsyncFunctionBase,
    A: Alloc,
    VD: VMDataTrait<Alloc= A>,
    ACTX: AsyncVMContext<VMData = VD>
{
    fn signature(&self, tyck_info_pool: &mut TyckInfoPool) -> Signature {
        <AFBase as AsyncFunctionBase>::signature(tyck_info_pool)
    }

    unsafe fn call_rtlc(&self, context: &ACTX, args: &[Value]) -> Result<Promise<A>, FFIException> {
        <AFBase as AsyncFunctionBase>::call_rtlc::<A, VD, ACTX>(context, args)
    }
}

pub struct AsyncResetGuard {
    wrapper_ptr: *mut Wrapper<()>,
    original: u8
}

impl Drop for AsyncResetGuard {
    fn drop(&mut self) {
        let wrapper_ref: &mut Wrapper<()> = unsafe { &mut *self.wrapper_ptr };
        wrapper_ref.ownership_info = unsafe { self.original }
    }
}

unsafe impl Send for AsyncResetGuard {}
unsafe impl Sync for AsyncResetGuard {}

pub struct AsyncShareGuard {
    wrapper_ptr: *mut Wrapper<()>
}

impl Drop for AsyncShareGuard {
    fn drop(&mut self) {
        let wrapper_ref: &mut Wrapper<()> = unsafe { &mut *self.wrapper_ptr };
        if wrapper_ref.ownership_info & OWN_INFO_GLOBAL_MASK == 0 {
            wrapper_ref.refcount -= 1;
            if wrapper_ref.refcount == 0 {
                wrapper_ref.ownership_info = wrapper_ref.ownership_info2;
            }
        }
    }
}

unsafe impl Send for AsyncShareGuard {}
unsafe impl Sync for AsyncShareGuard {}

pub trait AsyncReturnType<A: Alloc> : Send + Sync {
    fn is_err(&self) -> bool;

    fn resolve(self, alloc: &mut A, dests: &[*mut Value]) -> Result<usize, ExceptionInner>;
}

pub type PromiseResult<A: Alloc> = Box<dyn AsyncReturnType<A>>;

pub struct Promise<A: Alloc>(pub Pin<Box<dyn Future<Output=PromiseResult<A>> + Send>>);

impl<A: Alloc> StaticBase<Promise<A>> for Void {
    fn type_name() -> String {
        "promise".to_string()
    }
}

pub use crate::ffi::sync_fn::{
    value_copy,
    value_copy_norm,
    value_move_out,
    value_move_out_check,
    value_move_out_check_norm,
    value_move_out_check_norm_noalias,
    value_move_out_norm,
    value_move_out_norm_noalias
};
use crate::data::tyck::TyckInfoPool;
use crate::vm::al31f::alloc::Alloc;

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
        if original & OWN_INFO_GLOBAL_MASK == 0 {
            if original & OWN_INFO_WRITE_MASK != 0 {
                (*wrapper_ptr).ownership_info2 = original;
                (*wrapper_ptr).ownership_info
                    = original & (OWN_INFO_READ_MASK | OWN_INFO_OWNED_MASK);
                (*wrapper_ptr).refcount = 1;
            } else {
                (*wrapper_ptr).refcount += 1;
            }
        }
        Ok((
            &*data_ptr,
            AsyncShareGuard { wrapper_ptr }
        ))
    } else {
        Err(FFIException::Unchecked(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_ref<CR>(
    value: Value
) -> Result<(CR, AsyncShareGuard), FFIException>
    where CR: GenericTypeRef
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
        Err(FFIException::Unchecked(UncheckedException::OwnershipCheckFailure {
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
        Err(FFIException::Unchecked(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_mut_ref<CR>(
    value: Value
) -> Result<(CR, AsyncResetGuard), FFIException>
    where CR: GenericTypeRef
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
        Err(FFIException::Unchecked(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}
