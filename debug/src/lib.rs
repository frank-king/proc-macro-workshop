use proc_macro::TokenStream;

use inner::CustomDebug;
use syn::{parse_macro_input, DeriveInput};

mod inner;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let custom_debug = CustomDebug::from_derive_input(input);
    let result = custom_debug
        .build()
        .unwrap_or_else(|err| err.to_compile_error());
    // eprintln!("{}", result.to_string());
    TokenStream::from(result)
}
