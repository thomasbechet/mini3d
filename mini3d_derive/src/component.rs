use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;
use syn::{DeriveInput, Result, Data, DataStruct, Fields, Error, FieldsNamed, Token, FieldsUnnamed, Attribute, Generics, DataEnum, Visibility};

use crate::serialize;

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(&input.ident, &input.vis, &input.attrs, &input.generics, fields),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => derive_tuple(&input.ident, &input.vis, &input.attrs, &input.generics, fields),
        Data::Enum(data) => derive_enum(&input.ident, &input.vis, &input.attrs, &input.generics, data),
        _ => Err(Error::new(Span::call_site(), "Union not supported")),
    }
}

struct ComponentMeta {
    name: String,
}

pub(crate) fn camelcase_to_snakecase(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && (name.chars().nth(i - 1).unwrap().is_lowercase() || name.chars().nth(i + 1).map(|c| c.is_lowercase()).unwrap_or(false)) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

impl ComponentMeta {

    fn new(ident: &Ident) -> Self {
        Self {
            name: camelcase_to_snakecase(&ident.to_string()),
        }
    }

    fn merge(&mut self, attribute: ComponentAttribute) -> Result<()> {
        match attribute {
            ComponentAttribute::Name(name) => {
                self.name = name.value();
            }
        }
        Ok(())
    }
}

enum ComponentAttribute {
    Name(syn::LitStr),
}

impl syn::parse::Parse for ComponentAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "name" {
            let _: Token![=] = input.parse()?;
            Ok(ComponentAttribute::Name(input.parse()?))
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported getter attribute, expected `name` or `vis`",
            ))
        }
    }
}

fn derive_struct(ident: &Ident, vis: &Visibility, attrs: &[Attribute], generics: &Generics, fields: &FieldsNamed) -> Result<TokenStream> {

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let mut meta = ComponentMeta::new(ident);
    for attribute in attrs {
        if attribute.path().is_ident("component") {
            meta.merge(attribute.parse_args::<ComponentAttribute>()?)?;
        }
    }

    let serialize = serialize::derive_struct(ident, vis, attrs, generics, fields)?;

    let component_name = meta.name;
    
    let q = quote!{
        #serialize
        impl mini3d::registry::component::Component for #ident #ty_generics #where_clause {
            const NAME: &'static str = #component_name;
        }
    };
    Ok(q)
}

pub(crate) fn derive_tuple(ident: &Ident, vis: &Visibility, attrs: &[Attribute], generics: &Generics, fields: &FieldsUnnamed) -> Result<TokenStream> {

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let mut meta = ComponentMeta::new(ident);
    for attribute in attrs {
        if attribute.path().is_ident("component") {
            meta.merge(attribute.parse_args::<ComponentAttribute>()?)?;
        }
    }

    let serialize = serialize::derive_tuple(ident, vis, attrs, generics, fields)?;

    let component_name = meta.name;
    
    let q = quote!{
        #serialize
        impl mini3d::registry::component::Component for #ident #ty_generics #where_clause {
            const NAME: &'static str = #component_name;
        }
    };
    Ok(q)
}

fn derive_enum(ident: &Ident, vis: &Visibility, attrs: &[Attribute], generics: &Generics, data: &DataEnum) -> Result<TokenStream> {

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let mut meta = ComponentMeta::new(ident);
    for attribute in attrs {
        if attribute.path().is_ident("component") {
            meta.merge(attribute.parse_args::<ComponentAttribute>()?)?;
        }
    }

    let serialize = serialize::derive_enum(ident, vis, attrs, generics, data)?;

    let component_name = meta.name;
    
    let q = quote!{
        #serialize
        impl mini3d::registry::component::Component for #ident #ty_generics #where_clause {
            const NAME: &'static str = #component_name;
        }
    };
    Ok(q)
}