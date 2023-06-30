use proc_macro2::{TokenStream, Span, Ident};
use quote::quote;
use syn::{DeriveInput, Result, Data, Error, Attribute, Generics, DataEnum, LitStr, Fields};

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Enum(data) => derive_enum(&input.ident, &input.attrs, &input.generics, data),
        _ => Err(Error::new(Span::call_site(), "Only enums can derive error")),
    }
}

pub(crate) fn derive_enum(ident: &Ident, _attrs: &[Attribute], generics: &Generics, data: &DataEnum) -> Result<TokenStream> {
    
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    
    let mut formats = Vec::new();

    for variant in &data.variants {

        // Parse format
        let mut format = None;
        for attr in &variant.attrs {
            if attr.path().is_ident("error") {
                format = Some(attr.parse_args::<LitStr>()?);
            }
        }

        if let Some(format) = format {
            // Generate format
            match &variant.fields {
                Fields::Named(fields) => {
                    let ident = &variant.ident;
                    let field_idents = fields.named.iter().map(|field| &field.ident).collect::<Vec<_>>();
                    formats.push(quote!{ Self::#ident { #(ref #field_idents),* } => core::write!(f, #format, #(#field_idents = #field_idents),*) })
                },
                Fields::Unnamed(fields) => {
                    let ident = &variant.ident;
                    let field_indices = fields.unnamed.iter().enumerate().map(|(i, _)| Ident::new(&format!("field{}", i), Span::call_site()) ).collect::<Vec<_>>();
                    formats.push(quote!{ Self::#ident(#(ref #field_indices),*) => core::write!(f, #format, #(#field_indices),*) })
                },
                Fields::Unit => {
                    let ident = &variant.ident;
                    formats.push(quote!{ Self::#ident => core::write!(f, #format) })
                },
            }
        } else {
            let ident = &variant.ident;
            formats.push(quote!{ Self::#ident => core::write!(f, #ident) })
        }
    }
    Ok(quote!{

        impl core::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(#formats),*
                }
            }
        }

        impl mini3d::ecs::system::SystemError for #ident #ty_generics #where_clause {}

        impl From<#ident #ty_generics> for Box<dyn mini3d::ecs::system::SystemError> #ty_generics #where_clause {
            fn from(error: #ident #ty_generics) -> Self {
                Box::new(error)
            }
        }
    })
}