use std::collections::HashMap;
use std::marker::PhantomPinned;

use xjbutil::void::Void;

use crate::data::Value;
use crate::data::traits::{ChildrenType, StaticBase};

#[repr(transparent)]
pub struct Object {
    pub(crate) fields: HashMap<String, Value>,
    _pin: PhantomPinned
}

impl Object {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            _pin: PhantomPinned
        }
    }
}

impl StaticBase<Object> for Void {
    fn type_name() -> String { "object".into() }

    #[inline] fn children(vself: *const Object) -> ChildrenType {
        unsafe {
            let iter = Box::new((*vself).fields.iter().map(|(_, value): (_, &Value)| *value));
            Some(iter)
        }
    }
}

#[cfg(not(feature = "al31f-builtin-ops"))]
mod ops {
    use std::ptr::NonNull;
    use unchecked_unwrap::UncheckedUnwrap;
    use xjbutil::boxed_slice;
    use xjbutil::unchecked::UncheckedCellOps;

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
            context.add_heap_managed(object);
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
                .get_ref_unchecked()
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

            if value.is_ref() { context.mark(value); }

            object_ref.ptr.as_mut()
                .fields
                .get_mut_ref_unchecked()
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
