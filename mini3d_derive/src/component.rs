use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Result, Token, Type, Visibility,
};

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(
            &input.ident,
            &input.vis,
            &input.attrs,
            &input.generics,
            fields,
        ),
        _ => Err(Error::new(Span::call_site(), "Only struct are supported")),
    }
}

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

struct StructFieldEntry {
    ident: Ident,
    ty: Type,
}

fn parse_struct_field_entries(
    fields: &FieldsNamed,
) -> Result<Vec<StructFieldEntry>> {
    let mut entries = Vec::new();

    // for field in &fields.named {
    //     let attributes = FieldAttributes::build(&field.attrs)?;
    //     entries.push(StructFieldEntry {
    //         ident: field.ident.as_ref().unwrap().clone(),
    //         ty: field.ty.clone(),
    //     });
    // }
    
    Ok(entries)
}

// fn generate_struct_field(entry: &StructFieldEntry) -> Result<TokenStream> {
// }

fn derive_struct(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    fields: &FieldsNamed,
) -> Result<TokenStream> {
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let mut meta = ComponentMeta::new(ident);
    for attribute in attrs {
        if attribute.path().is_ident("component") {
            meta.merge(attribute.parse_args::<ComponentAttribute>()?)?;
        }
    }

    let name = meta.name;

    let q = quote! {

        impl mini3d_ecs2::component::NamedComponent for MyComponent {
            const IDENT: &'static str = #name;
        }

    };
    Ok(q)
}
