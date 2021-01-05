use proc_macro::TokenStream;

use quote::quote;
use std::iter::FromIterator;
use syn::export::TokenStream2;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Ident, Type};

macro_rules! tokenize (
    ( $ty:ident( $fmt:literal, $value:expr ) ) => {
        tokenize!($ty($fmt, $value, Span::call_site()));
    };
    ( $ty:ident( $fmt:literal, $value:expr, $span:expr ) ) => {
        $ty::new(format!($fmt, $value).as_str(), $span)
    }
);

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    if let Data::Struct(r#struct) = input.data {
        if let Fields::Named(named) = r#struct.fields {
            let fields: Vec<(Ident, Type)> = named
                .named
                .into_iter()
                .map(|field| (field.ident.unwrap(), field.ty))
                .collect();
            let option = quote!(std::option::Option);
            let some = quote!(std::option::Option::Some);
            let none = quote!(std::option::Option::None);
            let builder_ident = tokenize!(Ident("{}Builder", ident.to_string(), ident.span()));
            let builder_fields = TokenStream2::from_iter(fields.iter().map(|field| {
                let (ident, ty) = field;
                quote!(#ident: #option<#ty>, )
            }));
            let builder_inits = TokenStream2::from_iter(fields.iter().map(|field| {
                let (ident, _) = field;
                quote!(#ident: #none, )
            }));
            let builder_setters = TokenStream2::from_iter(fields.iter().map(|field| {
                let (ident, ty) = field;
                quote! {
                    pub fn #ident(&mut self, value: #ty) -> &mut Self {
                        self.#ident = Some(value);
                        self
                    }
                }
            }));
            let build_cond = TokenStream2::from_iter(fields.iter().map(|field| {
                let (ident, _) = field;
                quote! {
                    && self.#ident.is_some()
                }
            }));
            let build = TokenStream2::from_iter(fields.iter().map(|field| {
                let (ident, _) = field;
                quote!(#ident: self.#ident.take().unwrap(), )
            }));
            let output = quote! {
                impl #ident {
                    pub fn builder() -> #builder_ident {
                        #builder_ident { #builder_inits }
                    }
                }

                pub struct #builder_ident {
                    #builder_fields
                }

                impl #builder_ident {
                    #builder_setters
                    pub fn build(&mut self) -> #option<#ident> {
                        if true #build_cond {
                            #some(#ident {
                                #build
                            })
                        } else {
                            #none
                        }
                    }
                }
            };
            // eprintln!("{}", output.to_string());
            TokenStream::from(output)
        } else {
            return TokenStream::from(
                Error::new(ident.span(), "only named struct is supported").to_compile_error(),
            );
        }
    } else {
        return TokenStream::from(
            Error::new(ident.span(), "only struct is supported").to_compile_error(),
        );
    }
}
