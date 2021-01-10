use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let ident_builder = Ident::new(&format!("{}Builder", ident), ident.span());
    quote! (
        impl #ident {
            pub fn builder() -> #ident_builder {
                #ident_builder
            }
        }

        pub struct #ident_builder;
    )
    .into()
}
