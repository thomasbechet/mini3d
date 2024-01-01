use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, FieldsUnnamed, Generics, Ident, Visibility};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Result};

pub(crate) fn derive(input: &DeriveInput) -> Result<TokenStream> {
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

fn derive_struct(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    fields: &FieldsNamed,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics mini3d_core::reflection::ReadProperty for #ident #ty_generics #where_clause {}
        impl #impl_generics mini3d_core::reflection::WriteProperty for #ident #ty_generics #where_clause {}
        impl #impl_generics mini3d_core::reflection::Reflect for #ident #ty_generics #where_clause {}
    })
}

fn derive_tuple(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    fields: &FieldsUnnamed,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics mini3d_core::reflection::ReadProperty for #ident #ty_generics #where_clause {}
        impl #impl_generics mini3d_core::reflection::WriteProperty for #ident #ty_generics #where_clause {}
        impl #impl_generics mini3d_core::reflection::Reflect for #ident #ty_generics #where_clause {}
    })
}

fn derive_enum(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    data: &DataEnum,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics mini3d_core::reflection::ReadProperty for #ident #ty_generics #where_clause {}
        impl #impl_generics mini3d_core::reflection::WriteProperty for #ident #ty_generics #where_clause {}
        impl #impl_generics mini3d_core::reflection::Reflect for #ident #ty_generics #where_clause {}
    })
}
