use quote::quote;
use std::iter::FromIterator;
use syn::export::{ToTokens, TokenStream2};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse2, Data, DeriveInput, Error, Field, Fields, GenericParam, Generics, LitStr, Token};

pub struct CustomDebug {
    input: DeriveInput,
}

impl CustomDebug {
    pub fn from_derive_input(input: DeriveInput) -> Self {
        Self { input }
    }

    pub fn build(&self) -> syn::Result<TokenStream2> {
        let debug = quote!(std::fmt::Debug);
        let formatter = quote!(std::fmt::Formatter<'_>);
        let result = quote!(std::result::Result);
        let error = quote!(std::fmt::Error);
        let ident = &self.input.ident;
        let write_fields = self.format()?;
        let fmt = quote! {
                fn fmt(&self, fmt: &mut #formatter) -> #result<(), #error> {
                    write!(fmt, "{} {{", stringify!(#ident))?;
                    #write_fields
                    write!(fmt, "}}")?;
                    Ok(())
                }
        };
        if let Some((params, wheres)) = self.extract_generics(&self.input.generics) {
            Ok(quote! {
                impl<#params> #debug for #ident<#params> where #wheres {
                    #fmt
                }
            })
        } else {
            Ok(quote!( impl #debug for #ident { #fmt } ))
        }
    }

    fn extract_generics(&self, generics: &Generics) -> Option<(TokenStream2, TokenStream2)> {
        if generics.params.is_empty() {
            return None;
        }
        let params = &generics.params;
        let wheres = generics
            .where_clause
            .as_ref()
            .map(|r#where| &r#where.predicates);
        let debug = quote!(std::fmt::Debug);
        let wheres = TokenStream2::from_iter(
            wheres
                .iter()
                .map(|r#where| r#where.to_token_stream())
                .chain(params.iter().filter_map(|param| {
                    if let GenericParam::Type(ty) = param {
                        let ident = &ty.ident;
                        Some(quote!(#ident: #debug,))
                    } else {
                        None
                    }
                })),
        );
        if wheres.is_empty() {
            return None;
        }
        Some((params.to_token_stream(), wheres))
    }

    fn format(&self) -> syn::Result<TokenStream2> {
        match &self.input.data {
            Data::Struct(r#struct) => match &r#struct.fields {
                Fields::Named(named) => self.format_fields(named.named.iter()),
                Fields::Unnamed(unnamed) => self.format_fields(unnamed.unnamed.iter()),
                Fields::Unit => Ok(quote! {}),
            },
            Data::Enum(r#enum) => Err(Error::new(r#enum.variants.span(), "enum is not supported")),
            Data::Union(r#union) => Err(Error::new(union.fields.span(), "union is not supported")),
        }
    }

    fn format_fields<'a, Iter>(&'a self, fields: Iter) -> syn::Result<TokenStream2>
    where
        Iter: Iterator<Item = &'a Field>,
    {
        let mut result = TokenStream2::new();
        let mut fields = fields.enumerate().peekable();
        while let Some((idx, field)) = fields.next() {
            let debug_format = Self::get_debug_format(field)?;
            let punct = if fields.peek().is_some() { ',' } else { ' ' };
            let format = match &field.ident {
                Some(ident) => {
                    let format = format!(" {{}}: {}{}", debug_format, punct);
                    quote!(write!(fmt, #format, stringify!(#ident), self.#ident)?;)
                }
                None => {
                    let format = format!(" {}{}", debug_format, punct);
                    quote!(write!(fmt, #format, self.#idx)?;)
                }
            };
            result.extend(format);
        }
        Ok(result)
    }

    fn get_debug_format(field: &Field) -> syn::Result<String> {
        if field.attrs.is_empty() {
            return Ok("{:?}".to_owned());
        }
        if field.attrs.len() > 1 {
            return Err(Error::new(field.span(), "Too much attributes"));
        }
        let attr = &field.attrs[0];
        struct Format(LitStr);
        impl Parse for Format {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let _eq: Token![=] = input.parse()?;
                let format = input.parse()?;
                Ok(Format(format))
            }
        }
        let format: Format = parse2(attr.tokens.clone())?;
        Ok(format.0.value())
    }
}
