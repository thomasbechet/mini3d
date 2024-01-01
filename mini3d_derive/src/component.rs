use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Result, Token, Visibility,
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
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => derive_tuple(
            &input.ident,
            &input.vis,
            &input.attrs,
            &input.generics,
            fields,
        ),
        Data::Enum(data) => derive_enum(
            &input.ident,
            &input.vis,
            &input.attrs,
            &input.generics,
            data,
        ),
        _ => Err(Error::new(Span::call_site(), "Union not supported")),
    }
}

struct ComponentMeta {
    name: String,
    // script_name: String,
    storage: ComponentStorage,
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
            // name: "CTY_".to_owned() + &ident.to_string(),
            name: camelcase_to_snakecase(&ident.to_string()),
            // script_name: camelcase_to_snakecase(&ident.to_string()),
            storage: ComponentStorage::Single,
        }
    }

    fn view_ref_quote(&self, ident: &Ident) -> (String, TokenStream) {
        let name = ident.to_string() + "ViewRef";
        match &self.storage {
            ComponentStorage::Single => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewRef<#ident> },
            ),
            ComponentStorage::Array(size) => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewRef<#ident> },
            ),
            ComponentStorage::List => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewRef<#ident> },
            ),
            ComponentStorage::Tag => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewRef<#ident> },
            ),
        }
    }

    fn view_mut_quote(&self, ident: &Ident) -> (String, TokenStream) {
        let name = ident.to_string() + "ViewMut";
        match &self.storage {
            ComponentStorage::Single => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewMut<#ident> },
            ),
            ComponentStorage::Array(size) => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewMut<#ident> },
            ),
            ComponentStorage::List => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewMut<#ident> },
            ),
            ComponentStorage::Tag => (
                name,
                quote! { mini3d_core::ecs::view::native::single::NativeSingleViewMut<#ident> },
            ),
        }
    }

    fn merge(&mut self, attribute: ComponentAttribute) -> Result<()> {
        match attribute {
            ComponentAttribute::Name(name) => {
                self.name = name.value();
            }
            ComponentAttribute::Storage(storage) => {
                self.storage = storage;
            }
        }
        Ok(())
    }
}

enum ComponentStorage {
    Single,
    Array(usize),
    List,
    Tag,
}

enum ComponentAttribute {
    Name(syn::LitStr),
    Storage(ComponentStorage),
}

impl syn::parse::Parse for ComponentAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "name" {
            let _: Token![=] = input.parse()?;
            Ok(ComponentAttribute::Name(input.parse()?))
        } else if arg_name == "storage" {
            let _: Token![=] = input.parse()?;
            let storage = input.parse::<syn::LitStr>()?.value();
            if storage == "single" {
                Ok(ComponentAttribute::Storage(ComponentStorage::Single))
            } else if storage == "array" {
                Ok(ComponentAttribute::Storage(ComponentStorage::Array(0)))
            } else if storage == "list" {
                Ok(ComponentAttribute::Storage(ComponentStorage::List))
            } else if storage == "tag" {
                Ok(ComponentAttribute::Storage(ComponentStorage::Tag))
            } else {
                return Err(Error::new_spanned(
                    storage,
                    "unsupported storage type, expected `single`, `array`, `list` or `tag`",
                ));
            }
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported attribute, expected `name` or `storage`",
            ))
        }
    }
}

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
        impl mini3d_core::ecs::component::Component for #ident #ty_generics #where_clause {}

        impl #ident #ty_generics #where_clause {
            pub const NAME: &'static str = #name;
        }
    };
    Ok(q)
}

pub(crate) fn derive_tuple(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    fields: &FieldsUnnamed,
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
        impl mini3d_core::ecs::component::Component for #ident #ty_generics #where_clause {}

        impl #ident #ty_generics #where_clause {
            pub const NAME: &'static str = #name;
        }
    };
    Ok(q)
}

fn derive_enum(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    data: &DataEnum,
) -> Result<TokenStream> {
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let mut meta = ComponentMeta::new(ident);
    for attribute in attrs {
        if attribute.path().is_ident("component") {
            meta.merge(attribute.parse_args::<ComponentAttribute>()?)?;
        }
    }

    let name = meta.name;
    // let script_name = meta.script_name;

    let q = quote! {
        impl mini3d_core::ecs::component::Component for #ident #ty_generics #where_clause {}

        impl #ident #ty_generics #where_clause {
            pub const NAME: &'static str = #name;
        }
    };
    Ok(q)
}
