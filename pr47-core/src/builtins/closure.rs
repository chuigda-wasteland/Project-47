use std::any::TypeId;

use smallvec::SmallVec;
use xjbutil::void::Void;

use crate::data::Value;
use crate::data::generic::GenericTypeVT;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::tyck::TyckInfoPool;

pub struct Closure {
    pub capture: SmallVec<[Value; 4]>,
    pub func_id: usize
}

impl Closure {
    pub fn new(capture: SmallVec<[Value; 4]>, func_id: usize) -> Self {
        Self { capture, func_id }
    }
}

impl StaticBase<Closure> for Void {
    #[inline] fn children(vself: *const Closure) -> ChildrenType {
        let vself: &Closure = unsafe { &*vself };
        Some(Box::new(vself.capture.iter().map(|v: &Value| unsafe { v.ptr_repr })))
    }
}

pub fn create_closure_vt(_tyck_info_pool: &mut TyckInfoPool) -> GenericTypeVT {
    use crate::data::generic::gen_impls;

    todo!()
}
