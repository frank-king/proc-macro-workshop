#![allow(dead_code)]

use proc_macro::TokenStream;
use quote::quote;
use std::iter::FromIterator;
use syn::export::{Span, ToTokens, TokenStream2};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parenthesized, parse2, parse_macro_input, Expr, Fields, Ident, ItemStruct, LitInt, Token, Type,
    Visibility,
};

macro_rules! tokenize (
    ( $ty:ident( $fmt:literal, $value:expr ) ) => {
        $ty::new(format!($fmt, $value).as_str(), Span::call_site())
    }
);

macro_rules! bytes (
    (@low $bits:expr) => {$bits / 8};
    (@high $bits:expr) => {($bits + 7) / 8};
);

fn extract_names_and_bits(fields: &Fields) -> Vec<(String, usize)> {
    fields
        .iter()
        .filter_map(|field| {
            if let Type::Path(path) = &field.ty {
                if let Some(path) = path.path.segments.last() {
                    let bits = path
                        .ident
                        .to_string()
                        .trim_start_matches('B')
                        .parse::<usize>()
                        .ok();
                    return bits.and_then(|bits| {
                        field.ident.as_ref().map(|field| (field.to_string(), bits))
                    });
                }
            }
            None
        })
        .collect()
}

fn get_set_bits(input: TokenStream2) -> TokenStream2 {
    struct Pattern {
        op: Ident,
        exprs: Punctuated<Expr, Token![,]>,
    }
    impl Parse for Pattern {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let op = input.parse()?;
            let content;
            let _paran = parenthesized!(content in input);
            let exprs = content.parse_terminated(Expr::parse)?;
            Ok(Self { op, exprs })
        }
    }
    let pattern: Pattern = parse2(input).unwrap();
    let op = &pattern.op;
    let get = |idx: usize| -> TokenStream2 {
        (&pattern.exprs).iter().nth(idx).unwrap().to_token_stream()
    };
    let data = get(0);
    let offset = get(1);
    let bits = get(2);
    let value = get(3);
    quote! {
        let start_idx = (#offset) / 8;
        let end_idx = (#offset + #bits) / 8;
        let start_bit = (#offset % 8) as u8;
        let end_bit = ((#offset + #bits) % 8) as u8;
        if start_idx == end_idx {
            #op(#data[start_idx], start_bit, end_bit, &mut #value);
        } else {
            #op(#data[start_idx], start_bit, 8, &mut #value);
            for idx in start_idx + 1..end_idx {
                #op(#data[idx], 0, 8, &mut #value);
            }
            if end_bit > 0 {
                #op(#data[end_idx], 0, end_bit, &mut #value);
            }
        }
    }
}

fn generate_accessors(
    vis: &Visibility,
    names_and_bits: Vec<(String, usize)>,
) -> (TokenStream2, usize) {
    let mut offset = 0;
    let tokens = TokenStream2::from_iter(names_and_bits.iter().map(|(field, bits)| {
        let set_field = tokenize!(Ident("set_{}", field));
        let get_field = tokenize!(Ident("get_{}", field));
        let offset_lit = tokenize!(LitInt("{}", offset));
        let bits_lit = tokenize!(LitInt("{}", bits));
        let field_ty = tokenize!(Ident("u{}", bits.next_power_of_two().max(8)));
        let set_bits =
            get_set_bits(quote!(set_bits(&mut self.data, #offset_lit, #bits_lit, value)));
        let get_bits = get_set_bits(quote!(get_bits(&self.data, #offset_lit, #bits_lit, value)));
        // eprintln!("{:?}", field_ty);
        let output = quote! {
            #vis fn #set_field(&mut self, value: #field_ty) {
                let mut value = reverse_bits(value as u64, #bits_lit);
                #set_bits
            }
            #vis fn #get_field(&self) -> #field_ty {
                let mut value = 0u64;
                #get_bits
                value as #field_ty
            }
        };
        offset += bits;
        output
    }));
    (tokens, bytes!(@high offset))
}

#[proc_macro_attribute]
pub fn bitfield(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let ident = item_struct.ident;
    let vis = item_struct.vis;
    let (accessors, bytes) = generate_accessors(&vis, extract_names_and_bits(&item_struct.fields));
    let size = tokenize!(LitInt("{}_usize", bytes));
    let output = quote! {
        #[repr(C)]
        #vis struct #ident {
            data: [u8; #size],
        }

        impl #ident {
            #vis fn new() -> Self { Self { data: [0; #size] } }
            #accessors

        }
    };
    // eprintln!("{}", output.to_string());

    TokenStream::from(output)
}
