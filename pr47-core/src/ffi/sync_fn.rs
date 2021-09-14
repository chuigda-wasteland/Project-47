use crate::data::Value;
use crate::data::exception::{UncheckedException, Exception};
use crate::data::traits::StaticBase;
use crate::data::wrapper::{OwnershipInfo, Wrapper, OWN_INFO_READ_MASK};
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
    SetOwnershipInfo(*mut Wrapper<()>, OwnershipInfo)
}

impl Drop for OwnershipGuard {
    #[inline(always)] fn drop(&mut self) {
        match self {
            OwnershipGuard::DoNothing => {},
            OwnershipGuard::SetOwnershipInfo(wrapper_ptr, ownership_info) => {
                unsafe {
                    (*wrapper_ptr).ownership_info = ownership_info as u8;
                }
            }
        }
    }
}

#[inline] pub unsafe fn container_into_ref<'a, T, CR>(
    value: Value
) -> Result<(CR, OwnershipGuard), FFIException>
    where T: 'static,
          CR: ContainerRef,
          Void: StaticBase<T>
{
    let wrapper_ptr: *mut Wrapper<()> = value.untagged_ptr_field() as *mut _;
    if wrapper_ptr.ownership_info & OWN_INFO_READ_MASK {
        if wrapper_ptr.ownership_info != OwnershipInfo::SharedToRust as u8 {
            let original: u8 = wrapper_ptr.ownership_info;
            wrapper_ptr.ownership_info = OwnershipInfo::SharedToRust as u8;
            Ok((
                CR::create_ref(wrapper_ptr),
                OwnershipGuard::SetOwnershipInfo(wrapper_ptr, original as OwnershipInfo)
            ))
        } else {
            Ok((CR::create_ref(wrapper_ptr), OwnershipGuard::DoNothing))
        }
    } else {
        Err(FFIException::Right(UncheckedException::OwnershipCheckFailure{
            ownership_info: wrapper_ptr.ownership_info,
            expected_mask: OWN_INFO_READ_MASK
        }))
    }
}

#[inline] pub unsafe fn value_into_ref<'a, T>(
    _value: Value
) -> Result<(&'a T, OwnershipGuardNorm), FFIException>
    where T: 'static,
          Void: StaticBase<T>
{
    todo!()
}
