mod attrs;
mod types;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, FnArg};

use crate::attrs::parse_function_bind_attrs;

#[proc_macro_attribute]
pub fn pr47_function_bind(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: ItemFn = parse_macro_input!(item as ItemFn);

    let attrs: Vec<String> = match parse_function_bind_attrs(attr) {
        Ok(attrs) => attrs,
        Err(e) => {
            return (quote!{
                compile_error!( #e ) ;
            }).into()
        }
    };

    let args: Vec<&FnArg> = item.sig.inputs.iter().collect::<_>();
    let ret = quote!{
        #item
    };

    ret.into()
}
