use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::Parser, Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed,
    Result, Token, Type, Visibility,
};

struct ComponentMeta {
    name: String,
}

impl ComponentMeta {
    fn new(ident: &Ident) -> Self {
        fn camelcase_to_snakecase(name: &str) -> String {
            let mut result = String::new();
            for (i, c) in name.chars().enumerate() {
                if c.is_uppercase() {
                    if i > 0
                        && (name.chars().nth(i - 1).unwrap().is_lowercase()
                            || name
                                .chars()
                                .nth(i + 1)
                                .map(|c| c.is_lowercase())
                                .unwrap_or(false))
                    {
                        result.push('_');
                    }
                    result.push(c.to_lowercase().next().unwrap());
                } else {
                    result.push(c);
                }
            }
            result
        }
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
                "unsupported attribute, expected `name`",
            ))
        }
    }
}

pub fn derive(ast: &mut DeriveInput) -> Result<TokenStream> {
    let mut meta = ComponentMeta::new(&ast.ident);
    for attribute in &ast.attrs {
        if attribute.path().is_ident("component") {
            meta.merge(attribute.parse_args::<ComponentAttribute>()?)?;
        }
    }

    let name = meta.name;
    let ident = &ast.ident;
    let mut named_fields = Vec::new();
    let mut find_fields = Vec::new();

    match ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref mut fields),
            ..
        }) => {
            for field in fields.named.iter_mut() {
                let ty = field.ty.clone();
                let ident = field.ident.clone().unwrap();
                // Replace field with Field type
                *field =
                    Field::parse_named.parse2(quote!(pub #ident: mini3d::db::field::Field<#ty>))?;
                // Build field constructors
                let name = ident.to_string();
                named_fields.push(quote!(<#ty as mini3d::db::field::FieldType>::named(#name)));
                find_fields.push(quote!(#ident: api.find_field(id, #name).unwrap()));
            }
            fields
                .named
                .push(Field::parse_named.parse2(quote!(_id: mini3d::db::database::ComponentHandle))?);
        }
        _ => return Err(Error::new(Span::call_site(), "Only struct are supported")),
    }

    Ok(quote! {
        #ast

        impl #ident {
            pub const NAME: &'static str = #name;

            pub fn id(&self) -> mini3d::db::database::ComponentHandle {
                self._id
            }

            pub fn create_component(api: &mut mini3d::api::API) -> Self {
                let id = api
                    .create_component(
                        #name,
                        &[
                            #(#named_fields),*
                        ],
                    )
                    .unwrap();
                Self::meta(api)
            }

            pub fn meta(api: &mini3d::api::API) -> Self {
                let id = api.find_component(#name).unwrap();
                Self {
                    _id: id,
                    #(#find_fields),*
                }
            }
        }
    })
}
