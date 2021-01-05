use crate::inner::BuilderImpl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod inner;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    TokenStream::from(
        match BuilderImpl::from_derive_input(parse_macro_input!(input as DeriveInput)) {
            Ok(builder) => builder.build(),
            Err(err) => err.to_compile_error(),
        },
    )
}
