use quote::quote;
use std::iter::FromIterator;
use syn::export::TokenStream2;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Field, Fields};

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
        Ok(quote! {
            impl #debug for #ident {
                fn fmt(&self, fmt: &mut #formatter) -> #result<(), #error> {
                    write!(fmt, "{} {{", stringify!(#ident))?;
                    #write_fields
                    write!(fmt, "}}")?;
                    Ok(())
                }
            }
        })
    }

    fn format(&self) -> syn::Result<TokenStream2> {
        match &self.input.data {
            Data::Struct(r#struct) => Ok(match &r#struct.fields {
                Fields::Named(named) => self.format_fields(named.named.iter()),
                Fields::Unnamed(unnamed) => self.format_fields(unnamed.unnamed.iter()),
                Fields::Unit => quote! {},
            }),
            Data::Enum(r#enum) => Err(Error::new(r#enum.variants.span(), "enum is not supported")),
            Data::Union(r#union) => Err(Error::new(union.fields.span(), "union is not supported")),
        }
    }

    fn format_fields<'a, Iter>(&'a self, fields: Iter) -> TokenStream2
    where
        Iter: Iterator<Item = &'a Field>,
    {
        let result =
            TokenStream2::from_iter(fields.enumerate().map(|(idx, field)| match &field.ident {
                Some(ident) => quote!(write!(fmt, " {}: {:?},", stringify!(#ident), self.#ident)?;),
                None => quote!(write!(fmt, " {:?},", self.#idx)?;),
            }));
        if result.is_empty() {
            return result;
        }
        quote!(#result write!(fmt, " ")?; )
    }
}
