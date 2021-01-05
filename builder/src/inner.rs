use quote::quote;
use std::iter::FromIterator;
use syn::export::TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse2, Attribute, Data, DeriveInput, Error, Fields, GenericArgument, Ident,
    LitStr, PathArguments, Token, Type,
};

macro_rules! tokenize (
    ( $ty:ident( $fmt:literal, $value:expr ) ) => {
        tokenize!($ty($fmt, $value, Span::call_site()));
    };
    ( $ty:ident( $fmt:literal, $value:expr, $span:expr ) ) => {
        $ty::new(format!($fmt, $value).as_str(), $span)
    }
);

enum FieldKind {
    Required,
    Optional,
    Repeated { ident: Ident, ty: Type },
}

struct Field {
    ident: Ident,
    ty: Type,
    kind: FieldKind,
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

fn vec() -> TokenStream2 {
    quote!(std::vec::Vec)
}

impl BuilderImpl {
    pub fn from_derive_input(input: DeriveInput) -> syn::Result<Self> {
        let name = input.ident;
        if let Data::Struct(r#struct) = input.data {
            if let Fields::Named(named) = r#struct.fields {
                let builder_name = tokenize!(Ident("{}Builder", name.to_string(), name.span()));
                let mut fields = vec![];
                for field in named.named.into_iter() {
                    fields.push(Self::extract_field(field)?);
                }
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

    fn extract_field(field: syn::Field) -> syn::Result<Field> {
        let ident = field.ident.unwrap();
        let ty = field.ty;
        if !field.attrs.is_empty() {
            let each = Self::get_attr_each(field.attrs)?;
            let inner = Self::extract_inner(&ty, |_| true)
                .ok_or_else(|| Error::new(ty.span(), "Not a container"))?;
            let kind = FieldKind::Repeated {
                ident: each,
                ty: inner,
            };
            Ok(Field { ident, ty, kind })
        } else {
            let (kind, ty) = match Self::extract_option(&ty) {
                Some(ty) => (FieldKind::Optional, ty),
                None => (FieldKind::Required, ty),
            };
            Ok(Field { ident, ty, kind })
        }
    }

    fn get_attr_each(mut attrs: Vec<Attribute>) -> syn::Result<Ident> {
        if attrs.len() > 1 {
            return Err(Error::new(attrs[1].span(), "Too much attributes"));
        }
        let attr = attrs.pop().unwrap();
        let name = &attr.path.segments.last().unwrap().ident;
        if name != "builder" {
            return Err(Error::new(name.span(), "Unrecognized attribute"));
        }
        struct Each {
            each: Ident,
            sym: LitStr,
        }
        impl Parse for Each {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let content;
                let _paran = parenthesized!(content in input);
                let each = content.parse()?;
                let _eq: Token![=] = content.parse()?;
                let sym = content.parse()?;
                Ok(Each { each, sym })
            }
        }
        let tokens = attr.tokens;
        let each: Each = parse2(tokens)?;
        if each.each != "each" {
            return Err(Error::new(each.each.span(), "Unrecognized attribute"));
        }
        let each = each.sym;
        Ok(tokenize!(Ident("{}", each.value(), each.span())))
    }

    fn extract_inner(ty: &Type, outer_filter: impl Fn(&Ident) -> bool) -> Option<Type> {
        if let Type::Path(path) = ty {
            if let Some(last) = path.path.segments.last() {
                if outer_filter(&last.ident) {
                    if let PathArguments::AngleBracketed(arguments) = &last.arguments {
                        if let Some(GenericArgument::Type(ty)) = arguments.args.first() {
                            return Some(ty.clone());
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_option(ty: &Type) -> Option<Type> {
        Self::extract_inner(ty, |ident| ident == "Option")
    }

    fn builder_struct(&self) -> TokenStream2 {
        let option = option();
        let vec = vec();
        let builder_name = &self.builder_name;
        let contents = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, ty, kind } = field;
            match kind {
                FieldKind::Required | FieldKind::Optional => quote!(#ident: #option<#ty>, ),
                FieldKind::Repeated { ty, .. } => quote!(#ident: #vec<#option<#ty>>, ),
            }
        }));
        quote! {
            pub struct #builder_name {
                #contents
            }
        }
    }

    fn builder_fn(&self) -> TokenStream2 {
        let none = none();
        let vec = vec();
        let builder_name = &self.builder_name;
        let contents = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, kind, .. } = field;
            match kind {
                FieldKind::Required | FieldKind::Optional => quote!(#ident: #none, ),
                FieldKind::Repeated { .. } => quote!(#ident: #vec::new(), ),
            }
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
            let Field { ident, ty, kind } = field;
            match kind {
                FieldKind::Required | FieldKind::Optional => quote! {
                    pub fn #ident(&mut self, value: #ty) -> &mut Self {
                        self.#ident = #some(value);
                        self
                    }
                },
                FieldKind::Repeated {
                    ident: each,
                    ty: inner,
                } => quote! {
                    pub fn #each(&mut self, value: #inner) -> &mut Self {
                        self.#ident.push(#some(value));
                        self
                    }
                },
            }
        }))
    }

    fn build_fn(&self) -> TokenStream2 {
        let option = option();
        let some = some();
        let none = none();
        let name = &self.name;
        let cond = TokenStream2::from_iter(self.fields.iter().filter_map(|field| {
            let Field { ident, kind, .. } = field;
            if let FieldKind::Required = kind {
                Some(quote! ( && self.#ident.is_some() ))
            } else {
                None
            }
        }));
        let values = TokenStream2::from_iter(self.fields.iter().map(|field| {
            let Field { ident, kind, .. } = field;
            match kind {
                FieldKind::Required => quote!(#ident: self.#ident.take().unwrap(), ),
                FieldKind::Optional => quote!(#ident: self.#ident.take(), ),
                FieldKind::Repeated { .. } => {
                    quote! {
                        #ident: self.#ident.iter_mut().map(|each| each.take().unwrap()).collect(),
                    }
                }
            }
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
