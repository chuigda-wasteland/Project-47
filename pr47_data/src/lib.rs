use std::any::TypeId;
use std::ptr::NonNull;
use std::collections::BTreeMap;

pub trait Pr47DynBase {
    fn get_type_id(&self) -> TypeId;

    fn cast(&self, dest_type_id: TypeId) -> Result<NonNull<()>, String> {
        if self.get_type_id() == dest_type_id {
            Ok(unsafe { NonNull::new_unchecked(self as *const Self as *mut Self as *mut ()) })
        } else {
            Err(format!("Cannot cast from type {:?} to {:?}", dest_type_id, self.get_type_id()))
        }
    }

    fn cast_mut(&mut self, dest_type_id: TypeId) -> Result<NonNull<()>, String> {
        if self.get_type_id() == dest_type_id {
            Ok(unsafe { NonNull::new_unchecked(self as *mut Self as *mut ()) })
        } else {
            Err(format!("Cannot cast from type {:?} to {:?}", dest_type_id, self.get_type_id()))
        }
    }
}

pub struct Pr47Object {
    pub gc_ref: bool,
    pub flex: BTreeMap<String, Box<dyn Pr47DynBase>>
}

impl Pr47Object {
    pub fn new() -> Self {
        Self {
            gc_ref: true,
            flex: BTreeMap::new()
        }
    }
}

impl Pr47DynBase for Pr47Object {
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Pr47Object>()
    }
}

pub struct Pr47Int {
    pub gc_ref: bool,
    pub data: i64
}

impl Pr47Int {
    pub fn new(data: i64) -> Self {
        Self {
            gc_ref: true,
            data
        }
    }
}

impl Pr47DynBase for Pr47Int {
    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Pr47Int>()
    }
}

pub trait Pr47CallbackFunc {
    fn call(&self, args: Vec<&mut dyn Pr47DynBase>)
            -> std::result::Result<Box<dyn Pr47DynBase>, String>;
}
