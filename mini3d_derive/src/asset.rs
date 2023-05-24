// use proc_macro2::TokenStream;
// use syn::{DeriveInput, Result};

// pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
//     input.
// }

use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{DeriveInput, Result, DataStruct, Data, Error, Fields, Token, FieldsNamed};

use crate::{component::camelcase_to_snakecase, serialize};

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct_named(input, fields),
        _ => Ok(quote!({})),
        // _ => Err(Error::new(Span::call_site(), "Only structs are supported"))
    }
}

struct AssetMeta {
    name: String,
}

impl AssetMeta {

    fn new(ident: &Ident) -> Self {
        Self {
            name: camelcase_to_snakecase(&ident.to_string()),
        }
    }

    fn merge(&mut self, attribute: AssetAttribute) -> Result<()> {
        match attribute {
            AssetAttribute::Name(name) => {
                self.name = name.value();
            }
        }
        Ok(())
    }
}

enum AssetAttribute {
    Name(syn::LitStr),
}

impl syn::parse::Parse for AssetAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "name" {
            let _: Token![=] = input.parse()?;
            Ok(AssetAttribute::Name(input.parse()?))
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported getter attribute, expected `name` or `vis`",
            ))
        }
    }
}

fn derive_struct_named(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {

    let ident = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut meta = AssetMeta::new(ident);
    for attribute in &input.attrs {
        if attribute.path().is_ident("asset") {
            meta.merge(attribute.parse_args::<AssetAttribute>()?)?;
        }
    }

    let serialize = serialize::derive_struct(ident, &input.vis, &input.attrs, &input.generics, fields)?;

    let asset_name = meta.name;
    
    let q = quote!{
        #serialize
        impl mini3d::registry::asset::Asset for #ident #ty_generics #where_clause {
            const NAME: &'static str = #asset_name;
        }
    };
    
    Ok(q)
}