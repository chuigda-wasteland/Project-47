use proc_macro::TokenStream;

#[proc_macro]
pub fn pr47_test_suite(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        let expanded: TokenStream = TokenStream::from(quote::quote! {
            compile_error!("this function does not accept any parameters")
        });
        expanded
    } else {
        let expanded: TokenStream = TokenStream::from(quote::quote! {
            compile_error!("this function does not accept any parameters")
        });
        expanded
    }
}
