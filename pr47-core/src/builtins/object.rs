use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use xjbutil::unchecked::UncheckedCellOps;

use xjbutil::void::Void;
use crate::data::generic::GenericTypeRef;

use crate::data::Value;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::wrapper::{OWN_INFO_OWNED_MASK, Wrapper};

#[repr(transparent)]
pub struct Object {
    pub(crate) fields: UnsafeCell<HashMap<String, Value>>
}

impl Object {
    pub fn new() -> Self {
        Self {
            fields: UnsafeCell::new(HashMap::new())
        }
    }
}

impl StaticBase<Object> for Void {
    fn type_name() -> String { "object".into() }

    #[inline] fn children(vself: *const Object) -> ChildrenType {
        unsafe {
            let iter = Box::new((*vself).fields.get_ref_unchecked()
                .iter()
                .map(|x| x.1.ptr_repr.clone()));
            Some(iter)
        }
    }
}

pub struct ObjectRef {
    pub ptr: NonNull<Object>
}

impl GenericTypeRef for ObjectRef {
    unsafe fn create_ref(wrapper_ptr: *mut Wrapper<()>) -> Self {
        if (*wrapper_ptr).ownership_info & OWN_INFO_OWNED_MASK != 0 {
            Self {
                ptr: NonNull::new_unchecked(&mut (*wrapper_ptr).data.owned as *mut _ as _)
            }
        } else {
            Self {
                ptr: NonNull::new_unchecked((*wrapper_ptr).data.ptr as _)
            }
        }
    }
}

#[cfg(not(feature = "al31f-builtin-ops"))]
mod ops {
    use std::ptr::NonNull;
    use unchecked_unwrap::UncheckedUnwrap;
    use xjbutil::boxed_slice;

    use crate::builtins::object::{Object, ObjectRef};
    use crate::data::tyck::{TyckInfo, TyckInfoPool};
    use crate::data::Value;
    use crate::ffi::{DataOption, FFIException, Signature};
    use crate::ffi::sync_fn::{container_into_ref_noalias, FunctionBase, value_into_ref_noalias, VMContext};

    pub struct CreateObjectBind();
    pub struct ObjectGetBind();
    pub struct ObjectPutBind();

    impl FunctionBase for CreateObjectBind {
        fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
            let object_type: NonNull<TyckInfo> = tyck_info_pool.get_object_type();

            Signature {
                func_type: tyck_info_pool.create_function_type(
                    &[], &[object_type], &[]
                ),
                param_options: boxed_slice![],
                ret_option: boxed_slice![DataOption::Move],
            }
        }

        unsafe fn call_rtlc<CTX: VMContext>(
            context: &mut CTX,
            args: &[Value],
            rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            Self::call_unchecked(context, args, rets)
        }

        unsafe fn call_unchecked<CTX: VMContext>(
            context: &mut CTX,
            _args: &[Value],
            rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            let object: Value = Value::new_owned(Object::new());
            context.add_heap_managed(object.ptr_repr);
            **rets.get_unchecked(0) = object;
            Ok(())
        }
    }

    impl FunctionBase for ObjectGetBind {
        fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
            let object_type: NonNull<TyckInfo> = tyck_info_pool.get_object_type();
            let string_type: NonNull<TyckInfo> = tyck_info_pool.get_string_type();
            let any_type: NonNull<TyckInfo> = tyck_info_pool.get_any_type();
            let nullable_any: NonNull<TyckInfo> = tyck_info_pool.create_nullable_type(any_type);

            Signature {
                func_type: tyck_info_pool.create_function_type(
                    &[object_type, string_type], &[nullable_any], &[]
                ),
                param_options: boxed_slice![DataOption::Share, DataOption::Share],
                ret_option: boxed_slice![DataOption::RawUntyped]
            }
        }

        unsafe fn call_rtlc<CTX: VMContext>(
            _context: &mut CTX,
            args: &[Value],
            rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            let object_ref: ObjectRef = container_into_ref_noalias(*args.get_unchecked(0))?;
            let field: &String = value_into_ref_noalias(*args.get_unchecked(1))?;
            let value = object_ref.ptr.as_ref()
                .fields
                .get(field)
                .map(|x| *x)
                .or(Some(Value::new_null()))
                .unchecked_unwrap();
            **rets.get_unchecked(0) = value;
            Ok(())
        }

        unsafe fn call_unchecked<CTX: VMContext>(
            context: &mut CTX,
            args: &[Value],
            rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            // TODO rewrite once we have real unchecked operation
            Self::call_rtlc(context, args, rets)
        }
    }

    impl FunctionBase for ObjectPutBind {
        fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
            let object_type: NonNull<TyckInfo> = tyck_info_pool.get_object_type();
            let string_type: NonNull<TyckInfo> = tyck_info_pool.get_string_type();
            let any_type: NonNull<TyckInfo> = tyck_info_pool.get_any_type();
            let nullable_any: NonNull<TyckInfo> = tyck_info_pool.create_nullable_type(any_type);

            Signature {
                func_type: tyck_info_pool.create_function_type(
                    &[object_type, string_type, nullable_any], &[], &[]
                ),
                param_options: boxed_slice![
                    DataOption::MutShare,
                    DataOption::Share,
                    DataOption::RawUntyped
                ],
                ret_option: boxed_slice![]
            }
        }

        unsafe fn call_rtlc<CTX: VMContext>(
            context: &mut CTX,
            args: &[Value],
            _rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            let mut object_ref: ObjectRef = container_into_ref_noalias(*args.get_unchecked(0))?;
            let field: &String = value_into_ref_noalias(*args.get_unchecked(1))?;
            let value: Value = *args.get_unchecked(2);

            if value.is_ref() {
                context.mark(value.ptr_repr);
            }

            object_ref.ptr.as_mut()
                .fields
                .insert(field.clone(), value);
            Ok(())
        }

        unsafe fn call_unchecked<CTX: VMContext>(
            context: &mut CTX,
            args: &[Value],
            rets: &[*mut Value]
        ) -> Result<(), FFIException> {
            // TODO rewrite once we have real unchecked operation
            Self::call_rtlc(context, args, rets)
        }
    }
}
