use std::collections::HashMap;
use std::ptr::NonNull;

use xjbutil::void::Void;
use crate::data::generic::GenericTypeRef;

use crate::data::Value;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::wrapper::{OWN_INFO_OWNED_MASK, Wrapper};

pub struct Object {
    pub(crate) fields: HashMap<String, Value>
}

impl Object {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new()
        }
    }
}

impl StaticBase<Object> for Void {
    fn type_name() -> String { "object".into() }

    #[inline] fn children(vself: *const Object) -> ChildrenType {
        unsafe {
            let iter = Box::new((*vself).fields.iter().map(|x| x.1.ptr_repr.clone()));
            Some(iter)
        }
    }
}

pub struct ObjectRef {
    pub(crate) ptr: NonNull<Object>
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

mod ops {
    use unchecked_unwrap::UncheckedUnwrap;
    use xjbutil::boxed_slice;
    use xjbutil::void::Void;

    use crate::builtins::object::{Object, ObjectRef};
    use crate::data::traits::StaticBase;
    use crate::data::tyck::TyckInfoPool;
    use crate::data::Value;
    use crate::ffi::{DATA_OPTION_NULLABLE_MASK, DataOption, FFIException, Signature};
    use crate::ffi::sync_fn::{container_into_ref_noalias, FunctionBase, value_into_ref_noalias, VMContext};

    pub struct CreateObjectBind();
    pub struct ObjectGetBind();
    pub struct ObjectPutBind();

    impl FunctionBase for CreateObjectBind {
        fn signature(tyck_info_pool: &mut TyckInfoPool) -> Signature {
            Signature {
                param_types: boxed_slice![],
                param_options: boxed_slice![],
                ret_type: boxed_slice![<Void as StaticBase<Object>>::tyck_info(tyck_info_pool)],
                ret_option: boxed_slice![DataOption::Move as u8],
                exceptions: boxed_slice![]
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
            Signature {
                param_types: boxed_slice![
                    <Void as StaticBase<Object>>::tyck_info(tyck_info_pool),
                    <Void as StaticBase<String>>::tyck_info(tyck_info_pool)
                ],
                param_options: boxed_slice![DataOption::Share as u8, DataOption::Share as u8],
                ret_type: boxed_slice![
                    tyck_info_pool.create_any_type()
                ],
                ret_option: boxed_slice![
                    DataOption::RawUntyped as u8 | DATA_OPTION_NULLABLE_MASK
                ],
                exceptions: boxed_slice![]
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
            Signature {
                param_types: boxed_slice![
                    <Void as StaticBase<Object>>::tyck_info(tyck_info_pool),
                    <Void as StaticBase<String>>::tyck_info(tyck_info_pool),
                    tyck_info_pool.create_any_type()
                ],
                param_options: boxed_slice![
                    DataOption::MutShare as u8,
                    DataOption::Share as u8,
                    DataOption::RawUntyped as u8 | DATA_OPTION_NULLABLE_MASK
                ],
                ret_type: boxed_slice![],
                ret_option: boxed_slice![],
                exceptions: boxed_slice![]
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
