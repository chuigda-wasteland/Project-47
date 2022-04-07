use proc_macro::{TokenStream, TokenTree};

pub(crate) fn parse_function_bind_attrs(attr: TokenStream) -> Result<Vec<String>, String> {
    let mut ret: Vec<String> = vec![];
    for token in attr.into_iter() {
        match token {
            TokenTree::Group(group) => {
                return Err(format!("unexpected function binder parameter: {}", group))
            }
            TokenTree::Ident(ident) => {
                ret.push(ident.to_string())
            }
            TokenTree::Literal(lit) => {
                ret.push(lit.to_string())
            }
            TokenTree::Punct(_) => {}
        }
    }
    Ok(ret)
}
