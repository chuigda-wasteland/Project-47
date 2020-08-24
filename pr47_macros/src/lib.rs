#[macro_use]
extern crate syn;

use quote::quote;
use syn::parse::{Parse, Result, ParseBuffer};
use syn::{parse_macro_input, Expr, Ident, Type};
use proc_macro::TokenStream;

#[allow(unused_imports)]
use syn::token::Token;
use pr47_data::Pr47DynBase;

struct CallWithVec {
    func_name: Ident,
    types: Vec<Type>,
    vec: Expr
}

impl Parse for CallWithVec {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self> {
        let func_name: Ident = input.parse()?;
        input.parse::<Token![;]>()?;
        let mut types: Vec<Type> = vec![];
        while !input.peek(Token![;]) {
            types.push(input.parse()?);
            let _ = input.parse::<Token![,]>();
        }
        input.parse::<Token![;]>()?;
        let vec: Expr = input.parse()?;

        Ok(Self {
            func_name, types, vec
        })
    }
}

#[proc_macro]
pub fn call_with_vec(input: TokenStream) -> TokenStream {
    let CallWithVec { func_name, types, vec } = parse_macro_input!(input);

    let mut expr_casts = Vec::new();
    for (i, ty) in types.iter().enumerate() {
        match ty {
            Type::Reference(ref_ty) => {
                let base_ty = &ref_ty.elem;
                if let Some(_) = ref_ty.mutability {
                    expr_casts.push(quote! {
                        unsafe {
                            #vec[#i].cast_mut(std::any::TypeId::of::<#base_ty>())?
                                    .cast::<#base_ty>()
                                    .as_mut()
                        }
                    })
                } else {
                    expr_casts.push(quote! {
                        unsafe {
                            #vec[#i].cast(std::any::TypeId::of::<#base_ty>())?
                                    .cast::<#base_ty>()
                                    .as_ref()
                        }
                    })
                }
            }
            _ => {} // TODO compile error
        };
    }

    let expanded = quote! {
        #func_name ( #(#expr_casts),* )
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
    }
}