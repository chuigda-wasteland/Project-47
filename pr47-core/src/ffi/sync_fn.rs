use crate::data::Value;
use crate::data::exception::{UncheckedException};
use crate::data::traits::StaticBase;
use crate::data::wrapper::{OwnershipInfo, Wrapper, OWN_INFO_READ_MASK, OWN_INFO_OWNED_MASK};
use crate::ffi::{FFIException, Signature};
use crate::util::mem::FatPointer;
use crate::util::void::Void;
use crate::data::container::ContainerRef;

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

pub enum OwnershipGuard {
    DoNothing,
    SetOwnershipInfo(*mut Wrapper<()>, u8)
}

impl OwnershipGuard {
    #[cfg_attr(not(debug_assertions), inline(always))] pub fn clear(&mut self) {
        *self = OwnershipGuard::DoNothing;
    }
}

impl Drop for OwnershipGuard {
    #[cfg_attr(not(debug_assertions), inline(always))] fn drop(&mut self) {
        match self {
            OwnershipGuard::DoNothing => {},
            OwnershipGuard::SetOwnershipInfo(wrapper_ptr, ownership_info) => {
                unsafe {
                    (**wrapper_ptr).ownership_info = *ownership_info;
                }
            }
        }
    }
}

#[inline] pub unsafe fn value_move_out_check(
    value: Value
) -> Result<OwnershipGuard, FFIException> {
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_OWNED_MASK != 0 {
        Ok(OwnershipGuard::SetOwnershipInfo(wrapper_ptr, original))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            ownership_info: original,
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
        Ok(OwnershipGuard::SetOwnershipInfo(wrapper_ptr, original))
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            ownership_info: original,
            expected_mask: OWN_INFO_OWNED_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_ref<'a, T>(
    value: Value
) -> Result<(&'a T, OwnershipGuard), FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.ptr_repr.ptr as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        let data_ptr: *const T = value.get_as_mut_ptr_norm() as *const T;
        if original != OwnershipInfo::SharedToRust as u8 {
            (*wrapper_ptr).ownership_info = OwnershipInfo::SharedToRust as u8;
            Ok((&*data_ptr, OwnershipGuard::SetOwnershipInfo(wrapper_ptr, original)))
        } else {
            Ok((&*data_ptr, OwnershipGuard::DoNothing))
        }
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            ownership_info: original,
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
            ownership_info: original,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn container_into_ref<CR>(
    value: Value
) -> Result<(CR, OwnershipGuard), FFIException>
    where CR: ContainerRef
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    let original: u8 = (*wrapper_ptr).ownership_info;
    if original & OWN_INFO_READ_MASK != 0 {
        if original != OwnershipInfo::SharedToRust as u8 {
            (*wrapper_ptr).ownership_info = OwnershipInfo::SharedToRust as u8;
            Ok((
                CR::create_ref(wrapper_ptr),
                OwnershipGuard::SetOwnershipInfo(wrapper_ptr, original)
            ))
        } else {
            Ok((CR::create_ref(wrapper_ptr), OwnershipGuard::DoNothing))
        }
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure {
            object: value,
            ownership_info: original,
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
            ownership_info: original,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_mut_ref<'a, T>(
    _value: Value
) -> Result<(&'a mut T, OwnershipGuard), FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    todo!()
}

#[inline] pub unsafe fn value_into_mut_ref_noalias<'a, T>(
    _value: Value
) -> Result<&'a mut T, FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    todo!()
}

#[inline] pub unsafe fn container_into_mut_ref<CR>(
    _value: Value
) -> Result<(CR, OwnershipGuard), FFIException>
    where CR: ContainerRef
{
    todo!()
}

#[inline] pub unsafe fn container_into_mut_ref_noalias<CR>(
    _value: Value
) -> Result<(CR, OwnershipGuard), FFIException>
    where CR: ContainerRef
{
    todo!()
}
