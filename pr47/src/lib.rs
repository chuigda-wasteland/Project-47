#[macro_use]
extern crate paste;
#[macro_use]
extern crate pr47_macros;

use paste::paste;
use memoffset::offset_of;

use pr47_data;

macro_rules! register_callbacks {
    ( $($name:ident ( $($types:tt)* ) ; )* ) => {
        $(
            paste! {
                struct [< Pr47CallbackFunc_ $name >] () ;

                impl [< Pr47CallbackFunc_ $name >] {
                    fn new() -> Self { Self () }
                }

                impl pr47_data::Pr47CallbackFunc for [< Pr47CallbackFunc_ $name >] {
                    fn call(&self, args: Vec<&mut dyn Pr47DynBase>)
                        -> std::result::Result<Box<dyn Pr47DynBase>, String> {
                        Ok( call_with_vec! ( $name ; $($types)* ; args ) )
                    }
                }
            }
        )*
    }
}

macro_rules! define_structs {
    ( $($name:ident { $($field:ident : $type:ty),* $(,)? })* ) => {
        $(
            #[repr(align(8))]
            struct $name {
                #[allow(dead_code)]
                pub object_id: TypeId,
                #[allow(dead_code)]
                pub gc_ref: u32,
                #[allow(dead_code)]
                pub flex: BTreeMap<String, String>,
                $(pub $field : $type,)*
            }

            impl $name {
                #[allow(dead_code)]
                pub fn new_intern($($field : $type),*) -> Self {
                    Self {
                        object_id: TypeId::of::<$name>(),
                        gc_ref: 0,
                        flex: BTreeMap::new(),
                        $($field : $field,)*
                    }
                }
            }

            paste! {
                lazy_static! {
                    static ref [<$name _PR47_FIELD_MAP>] : BTreeMap<String, (TypeId, usize)> = {
                        let mut ret = BTreeMap::new();
                        $(ret.insert(stringify!($field).into(),
                                     (TypeId::of::<$type>(), offset_of!($name, $field)));)*
                        ret
                    };
                }
            }
        )*
    }
}

use pr47_data::{ Pr47Int, Pr47DynBase };
fn add(a: &Pr47Int, b: &Pr47Int) -> Box<dyn Pr47DynBase> {
    Box::new(Pr47Int::new(a.data + b.data))
}

register_callbacks!(
    add ( &Pr47Int , &Pr47Int ) ;
);

#[cfg(test)]
mod tests {
    use pr47_data::{ Pr47Int, Pr47DynBase };
    use pr47_data::Pr47CallbackFunc;
    use crate::Pr47CallbackFunc_add;
    use std::any::TypeId;

    #[test]
    fn it_works() {
        let callback = Pr47CallbackFunc_add::new();

        let mut args = vec![
            Pr47Int::new(12),
            Pr47Int::new(24)
        ];
        let args_ref =
            args.iter_mut()
                .map(|ptr| ptr as &mut dyn Pr47DynBase )
                .collect::<Vec<_>>();

        let result: Box<dyn Pr47DynBase> = callback.call(args_ref).unwrap();

        assert_eq!(unsafe {
            result.cast(TypeId::of::<Pr47Int>()).unwrap().cast::<Pr47Int>().as_ref()
        }.data, 36);
    }
}
