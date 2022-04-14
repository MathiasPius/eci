use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Component)]
pub fn derive_answer_fn(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = &input.ident;
    let name = ident.to_string();

    TokenStream::from(quote! {
        impl Component for #ident {
            const COMPONENT_TYPE: &'static str = #name;
        }
    })
}
