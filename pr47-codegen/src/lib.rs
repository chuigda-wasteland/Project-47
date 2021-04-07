use proc_macro::TokenStream;

#[proc_macro]
pub fn pr47_test_suite(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        let expanded = quote::quote! {
            compile_error!("this function does not accept any parameters")
        };
        TokenStream::from(expanded)
    } else {
        let expanded = quote::quote! {
            compile_error!("this function does not accept any parameters")
        };
        TokenStream::from(expanded)
    }
}
