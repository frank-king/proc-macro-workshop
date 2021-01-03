use proc_macro::TokenStream;
use quote::quote;
use syn::export::Span;
use syn::{parse_macro_input, ItemStruct, Type};

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item_struct = parse_macro_input!(input as ItemStruct);
    let ident = item_struct.ident;

    let bits: usize = item_struct
        .fields
        .iter()
        .map(|field| {
            if let Type::Path(path) = &field.ty {
                if let Some(path) = path.path.segments.last() {
                    let name = path.ident.to_string();
                    return name.trim_start_matches('B').parse::<usize>().unwrap_or(0);
                }
            }
            0
        })
        .sum();
    let size = (bits + 7) / 8;
    let size = syn::LitInt::new(format!("{}_usize", size).as_str(), Span::call_site());
    TokenStream::from(quote! {
        #[repr(C)]
        struct #ident {
            data: [u8; #size],
        }
    })
}
