use quote::quote;
use std::iter::FromIterator;
use syn::export::TokenStream2;
use syn::{Data, DeriveInput, Error, Fields, Ident, Result, Type};

macro_rules! tokenize (
    ( $ty:ident( $fmt:literal, $value:expr ) ) => {
        tokenize!($ty($fmt, $value, Span::call_site()));
    };
    ( $ty:ident( $fmt:literal, $value:expr, $span:expr ) ) => {
        $ty::new(format!($fmt, $value).as_str(), $span)
    }
);

struct Field {
    ident: Ident,
    ty: Type,
}

pub struct BuilderImpl {
    name: Ident,
    builder_name: Ident,
    fields: Vec<Field>,
}

fn option() -> TokenStream2 {
    quote!(std::option::Option)
}

fn some() -> TokenStream2 {
    quote!(std::option::Option::Some)
}

fn none() -> TokenStream2 {
    quote!(std::option::Option::None)
}

impl BuilderImpl {
    pub fn from_derive_input(input: DeriveInput) -> Result<Self> {
        let name = input.ident;
        if let Data::Struct(r#struct) = input.data {
            if let Fields::Named(named) = r#struct.fields {
                let builder_name = tokenize!(Ident("{}Builder", name.to_string(), name.span()));
                let fields: Vec<Field> = named
                    .named
                    .into_iter()
                    .map(|field| Field {
                        ident: field.ident.unwrap(),
                        ty: field.ty,
                    })
                    .collect();
                return Ok(Self {
                    name,
                    builder_name,
                    fields,
                });
            }
        }
        Err(Error::new(name.span(), "only named struct is supported"))
    }

    pub fn build(&self) -> TokenStream2 {
        let builder_fn = self.builder_fn();
        let builder_struct = self.builder_struct();
        let builder_setters = self.builder_setters();
        let build_fn = self.build_fn();
        let name = &self.name;
        let builder_name = &self.builder_name;
        quote! {
            impl #name {
                #builder_fn
            }

            #builder_struct

            impl #builder_name {
                #builder_setters
                #build_fn
            }
        }
    }

    fn builder_struct(&self) -> TokenStream2 {
        let option = option();
        let builder_name = &self.builder_name;
        let contents = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, ty } = field;
            quote!(#ident: #option<#ty>, )
        }));
        quote! {
            pub struct #builder_name {
                #contents
            }
        }
    }

    fn builder_fn(&self) -> TokenStream2 {
        let none = none();
        let builder_name = &self.builder_name;
        let contents = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, ty: _ } = field;
            quote!(#ident: #none, )
        }));
        quote! {
            pub fn builder() -> #builder_name {
                #builder_name{ #contents }
            }
        }
    }

    fn builder_setters(&self) -> TokenStream2 {
        let some = some();
        TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, ty } = field;
            quote! {
                pub fn #ident(&mut self, value: #ty) -> &mut Self {
                    self.#ident = #some(value);
                    self
                }
            }
        }))
    }

    fn build_fn(&self) -> TokenStream2 {
        let option = option();
        let some = some();
        let none = none();
        let name = &self.name;
        let cond = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, ty: _ } = field;
            quote! {
                && self.#ident.is_some()
            }
        }));
        let values = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, ty: _ } = field;
            quote!(#ident: self.#ident.take().unwrap(), )
        }));
        quote! {
            pub fn build(&mut self) -> #option<#name> {
                if true #cond {
                    #some(#name {
                        #values
                    })
                } else {
                    #none
                }
            }
        }
    }
}
