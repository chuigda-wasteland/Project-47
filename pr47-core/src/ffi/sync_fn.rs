use xjbutil::fat_ptr::FatPointer;
use xjbutil::void::Void;

use crate::data::Value;
use crate::data::container::ContainerRef;
use crate::data::exception::{UncheckedException};
use crate::data::traits::StaticBase;
use crate::data::wrapper::{OwnershipInfo, Wrapper};
use crate::data::wrapper::{
    OWN_INFO_OWNED_MASK,
    OWN_INFO_READ_MASK,
    OWN_INFO_WRITE_MASK
};
use crate::ffi::{FFIException, Signature};

pub trait VMContext: 'static + Sized {
    fn allocate(&mut self, fat_ptr: FatPointer);
    fn mark(&mut self, fat_ptr: FatPointer);
}

pub trait FunctionBase: 'static {
    fn signature() -> Signature;

    fn call_tyck<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException>;

    unsafe fn call_rtlc<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException>;

    unsafe fn call_unchecked<CTX: VMContext>(
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException>;
}

pub trait Function<CTX: VMContext>: 'static {
    fn signature(&self) -> Signature;

    fn call_tyck(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException>;

    unsafe fn call_rtlc(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException>;

    unsafe fn call_unchecked(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException>;
}

impl<FBase, CTX> Function<CTX> for FBase where
    FBase: FunctionBase,
    CTX: VMContext
{
    #[inline] fn signature(&self) -> Signature {
        <FBase as FunctionBase>::signature()
    }

    #[inline] fn call_tyck(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        <FBase as FunctionBase>::call_tyck(context, args, rets)
    }

    #[inline] unsafe fn call_rtlc(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        <FBase as FunctionBase>::call_rtlc(context, args, rets)
    }

    #[inline] unsafe fn call_unchecked(
        &self,
        context: &mut CTX,
        args: &[Value],
        rets: &[*mut Value]
    ) -> Result<(), FFIException> {
        <FBase as FunctionBase>::call_unchecked(context, args, rets)
    }
}

pub struct OwnershipGuard {
    wrapper_ptr: *mut Wrapper<()>,
    ownership_info: u8
}

impl OwnershipGuard {
    #[inline(always)]
    pub fn new(wrapper_ptr: *mut Wrapper<()>, ownership_info: u8) -> Self {
        Self { wrapper_ptr, ownership_info }
    }
}

impl Drop for OwnershipGuard {
    #[cfg_attr(not(debug_assertions), inline(always))] fn drop(&mut self) {
        unsafe {
            (*self.wrapper_ptr).ownership_info = self.ownership_info;
        }
    }
}

#[inline] pub unsafe fn value_move_out_check(
    value: Value
) -> Result<OwnershipGuard, FFIException> {
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_OWNED_MASK != 0 {
        Ok(OwnershipGuard::new(wrapper_ptr, original))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_OWNED_MASK
        }))
    }
}

#[inline] pub unsafe fn value_move_out_check_norm(
    value: Value
) -> Result<OwnershipGuard, FFIException> {
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_OWNED_MASK != 0 {
        Ok(OwnershipGuard::new(wrapper_ptr, original))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_OWNED_MASK
        }))
    }
}

#[inline] pub unsafe fn value_move_out<T>(value: Value) -> T
    where T: 'static,
          Void: StaticBase<T>
{
    value.move_out::<T>()
}

#[inline] pub unsafe fn value_move_out_norm<T>(value: Value) -> T
    where T: 'static,
          Void: StaticBase<T>
{
    value.move_out_norm::<T>()
}

#[inline] pub unsafe fn value_into_ref<'a, T>(
    value: Value
) -> Result<(&'a T, Option<OwnershipGuard>), FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        let data_ptr: *const T = value.get_as_mut_ptr_norm() as *const T;
        if original != OwnershipInfo::SharedToRust as u8 {
            (*wrapper_ptr).ownership_info = OwnershipInfo::SharedToRust as u8;
            Ok((
                &*data_ptr,
                Some(OwnershipGuard::new(wrapper_ptr, original))
            ))
        } else {
            Ok((&*data_ptr, None))
        }
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_ref_noalias<'a, T>(
    value: Value
) -> Result<&'a T, FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        let data_ptr: *const T = value.get_as_mut_ptr_norm() as *const T;
        Ok(&*data_ptr)
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_ref<CR>(
    value: Value
) -> Result<(CR, Option<OwnershipGuard>), FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        if original != OwnershipInfo::SharedToRust as u8 {
            (*wrapper_ptr).ownership_info = OwnershipInfo::SharedToRust as u8;
            Ok((
                CR::create_ref(wrapper_ptr),
                Some(OwnershipGuard::new(wrapper_ptr, original))
            ))
        } else {
            Ok((CR::create_ref(wrapper_ptr), None))
        }
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_ref_noalias<CR>(
    value: Value
) -> Result<CR, FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        Ok(CR::create_ref(wrapper_ptr))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_mut_ref<'a, T>(
    value: Value
) -> Result<(&'a mut T, OwnershipGuard), FFIException>
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
            OwnershipGuard::new(wrapper_ptr, original)
        ))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_mut_ref_noalias<'a, T>(
    value: Value
) -> Result<&'a mut T, FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_WRITE_MASK != 0 {
        let data_ptr: *mut T = value.get_as_mut_ptr_norm() as *mut T;
        Ok(&mut *data_ptr)
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_mut_ref<CR>(
    value: Value
) -> Result<(CR, OwnershipGuard), FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_WRITE_MASK != 0 {
        (*wrapper_ptr).ownership_info = OwnershipInfo::MutSharedToRust as u8;
        Ok((
            CR::create_ref(wrapper_ptr),
            OwnershipGuard::new(wrapper_ptr, original)
        ))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_mut_ref_noalias<CR>(
    value: Value
) -> Result<CR, FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_WRITE_MASK != 0 {
        Ok(CR::create_ref(wrapper_ptr))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_WRITE_MASK
        }))
    }
}

#[inline] pub unsafe fn value_copy<T>(value: Value) -> Result<T, FFIException>
    where T: 'static + Clone,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        let data_ptr: *const T = value.get_as_mut_ptr_norm() as *const T;
        Ok((&*data_ptr).clone())
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn value_copy_norm<T>(value: Value) -> Result<T, FFIException>
    where T: 'static + Clone,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        let data_ptr: *const T = value.get_as_mut_ptr_norm() as *const T;
        Ok((&*data_ptr).clone())
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}
