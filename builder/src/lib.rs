use crate::inner::BuilderImpl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod inner;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let result = match BuilderImpl::from_derive_input(parse_macro_input!(input as DeriveInput)) {
        Ok(builder) => builder.build(),
        Err(err) => err.to_compile_error(),
    };
    // eprintln!("{}", result.to_string());
    TokenStream::from(result)
}
