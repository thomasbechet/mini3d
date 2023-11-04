use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, FieldsUnnamed, Generics, Ident, ImplGenerics, Index, Token, Type,
    TypeGenerics, Variant, Visibility, WhereClause,
};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Result};

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

pub const fn fnv1a_hash_32(string: &str) -> u32 {
    const FNV1A_HASH_32: u32 = 0x811c9dc5;
    const FNV1A_PRIME_32: u32 = 0x01000193;
    let bytes = string.as_bytes();
    let mut hash = FNV1A_HASH_32;
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(FNV1A_PRIME_32);
        i += 1;
    }
    hash
}

fn parse_version(input: syn::parse::ParseStream) -> syn::Result<(u8, u8, u8)> {
    let major: syn::LitInt = input.parse()?;
    let _: Token![.] = input.parse()?;
    let minor: syn::LitInt = input.parse()?;
    let _: Token![.] = input.parse()?;
    let patch: syn::LitInt = input.parse()?;
    Ok((
        major.base10_parse()?,
        minor.base10_parse()?,
        patch.base10_parse()?,
    ))
}

/// Attributes ///
enum StructAttribute {
    Version((u8, u8, u8)),
}

impl syn::parse::Parse for StructAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "version" {
            let _: Token![=] = input.parse()?;
            Ok(StructAttribute::Version(parse_version(input)?))
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported serialize attribute, expected `version`, `skip` or `since`",
            ))
        }
    }
}

struct StructAttributes {
    version: (u8, u8, u8),
    removed_fields: Vec<(Ident, Type)>,
}

enum TupleAttribute {
    Version((u8, u8, u8)),
}

impl syn::parse::Parse for TupleAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "version" {
            let _: Token![=] = input.parse()?;
            Ok(TupleAttribute::Version(parse_version(input)?))
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported serialize attribute, expected `version`, `skip` or `since`",
            ))
        }
    }
}

impl StructAttributes {
    fn new() -> Self {
        Self {
            version: (0, 0, 0),
            removed_fields: Vec::new(),
        }
    }

    fn parse(attributes: &[Attribute]) -> Result<Self> {
        let mut struct_attributes = Self::new();
        for attribute in attributes {
            if attribute.path().is_ident("serialize") {
                let attribute = attribute.parse_args::<StructAttribute>()?;
                match attribute {
                    StructAttribute::Version(version) => {
                        struct_attributes.version = version;
                    }
                }
            }
        }
        Ok(struct_attributes)
    }
}

struct TupleAttributes {
    version: (u8, u8, u8),
    removed_fields: Vec<(usize, Type)>,
}

impl TupleAttributes {
    fn new() -> Self {
        Self {
            version: (0, 0, 0),
            removed_fields: Vec::new(),
        }
    }

    fn parse(attributes: &[Attribute]) -> Result<Self> {
        let mut struct_attributes = Self::new();
        for attribute in attributes {
            if attribute.path().is_ident("serialize") {
                let attribute = attribute.parse_args::<StructAttribute>()?;
                match attribute {
                    StructAttribute::Version(version) => {
                        struct_attributes.version = version;
                    }
                }
            }
        }
        Ok(struct_attributes)
    }
}

enum EnumAttribute {
    Version((u8, u8, u8)),
}

impl syn::parse::Parse for EnumAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "version" {
            let _: Token![=] = input.parse()?;
            Ok(EnumAttribute::Version(parse_version(input)?))
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported serialize attribute, expected `version`, `skip` or `since`",
            ))
        }
    }
}

struct EnumAttributes {
    version: (u8, u8, u8),
}

impl EnumAttributes {
    fn new() -> Self {
        Self { version: (0, 0, 0) }
    }

    fn parse(attributes: &[Attribute]) -> Result<Self> {
        let mut struct_attributes = Self::new();
        for attribute in attributes {
            if attribute.path().is_ident("serialize") {
                let attribute = attribute.parse_args::<EnumAttribute>()?;
                match attribute {
                    EnumAttribute::Version(version) => {
                        struct_attributes.version = version;
                    }
                }
            }
        }
        Ok(struct_attributes)
    }
}

enum FieldAttribute {
    Skip,
    Since((u8, u8, u8)),
    Default(Expr),
}

impl syn::parse::Parse for FieldAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name == "skip" {
            Ok(FieldAttribute::Skip)
        } else if arg_name == "since" {
            let _: Token![=] = input.parse()?;
            Ok(FieldAttribute::Since(parse_version(input)?))
        } else if arg_name == "default" {
            let _: Token![=] = input.parse()?;
            Ok(FieldAttribute::Default(input.parse()?))
        } else {
            Err(Error::new_spanned(
                arg_name,
                "unsupported serialize attribute, expected `version`, `skip` or `since`",
            ))
        }
    }
}

struct FieldAttributes {
    skip: bool,
    since: Option<(u8, u8, u8)>,
    default: Option<Expr>,
}

impl FieldAttributes {
    fn new() -> Self {
        Self {
            skip: false,
            since: None,
            default: None,
        }
    }

    fn skip() -> Self {
        Self {
            skip: true,
            since: None,
            default: None,
        }
    }

    fn build(attributes: &[Attribute]) -> Result<Self> {
        for attribute in attributes {
            if attribute.path().is_ident("serialize") {
                return attribute.parse_args::<FieldAttributes>();
            }
        }
        Ok(Self::new())
    }
}

impl Parse for FieldAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut field_attributes = Self::new();
        loop {
            if input.is_empty() {
                break;
            }
            let attribute = input.parse::<FieldAttribute>()?;
            match attribute {
                FieldAttribute::Skip => field_attributes.skip = true,
                FieldAttribute::Since(version) => field_attributes.since = Some(version),
                FieldAttribute::Default(expr) => field_attributes.default = Some(expr),
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
        }
        Ok(field_attributes)
    }
}

/// ENTRIES ///

struct StructFieldEntry {
    ident: Ident,
    ty: Type,
    attributes: FieldAttributes,
}

struct TupleFieldEntry {
    index: usize,
    ty: Type,
    attributes: FieldAttributes,
}

enum EnumFieldEntry {
    Struct {
        ident: Ident,
        hash: u32,
        entries: Vec<StructFieldEntry>,
    },
    Tuple {
        ident: Ident,
        hash: u32,
        entries: Vec<TupleFieldEntry>,
    },
    Unit {
        ident: Ident,
        hash: u32,
    },
}

fn parse_struct_field_entries(
    attributes: &StructAttributes,
    fields: &FieldsNamed,
) -> Result<Vec<StructFieldEntry>> {
    let mut entries = Vec::new();

    for field in &fields.named {
        let attributes = FieldAttributes::build(&field.attrs)?;
        entries.push(StructFieldEntry {
            ident: field.ident.as_ref().unwrap().clone(),
            ty: field.ty.clone(),
            attributes,
        });
    }

    for (field_ident, field_type) in &attributes.removed_fields {
        entries.push(StructFieldEntry {
            ident: field_ident.clone(),
            ty: field_type.clone(),
            attributes: FieldAttributes::skip(),
        });
    }

    // Sort entries by name
    entries.sort_by(|a, b| a.ident.cmp(&b.ident));

    Ok(entries)
}

fn parse_tuple_field_entries(
    attributes: &TupleAttributes,
    fields: &FieldsUnnamed,
) -> Result<Vec<TupleFieldEntry>> {
    let mut entries = Vec::new();

    for (index, field) in fields.unnamed.iter().enumerate() {
        let attributes = FieldAttributes::build(&field.attrs)?;
        entries.push(TupleFieldEntry {
            index,
            ty: field.ty.clone(),
            attributes,
        });
    }

    for (index, field_type) in &attributes.removed_fields {
        entries.push(TupleFieldEntry {
            index: *index,
            ty: field_type.clone(),
            attributes: FieldAttributes::skip(),
        });
    }

    // Sort entries by index
    entries.sort_by(|a, b| a.index.cmp(&b.index));

    Ok(entries)
}

fn parse_enum_field_entries(
    attributes: &EnumAttributes,
    variants: &Punctuated<Variant, Token![,]>,
) -> Result<Vec<EnumFieldEntry>> {
    let mut entries = Vec::new();

    for variant in variants {
        match variant.fields {
            Fields::Named(ref fields) => {
                let struct_attributes = StructAttributes::parse(&variant.attrs)?;
                let struct_entries = parse_struct_field_entries(&struct_attributes, fields)?;
                entries.push(EnumFieldEntry::Struct {
                    ident: variant.ident.clone(),
                    hash: fnv1a_hash_32(&variant.ident.to_string()),
                    entries: struct_entries,
                });
            }
            Fields::Unnamed(ref fields) => {
                let tuple_attributes = TupleAttributes::parse(&variant.attrs)?;
                let tuple_entries = parse_tuple_field_entries(&tuple_attributes, fields)?;
                entries.push(EnumFieldEntry::Tuple {
                    ident: variant.ident.clone(),
                    hash: fnv1a_hash_32(&variant.ident.to_string()),
                    entries: tuple_entries,
                });
            }
            Fields::Unit => {
                entries.push(EnumFieldEntry::Unit {
                    ident: variant.ident.clone(),
                    hash: fnv1a_hash_32(&variant.ident.to_string()),
                });
            }
        }
    }

    Ok(entries)
}

fn generate_header(
    header_type_ident: &Ident,
    vis: &Visibility,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    field_idents: &[Ident],
    field_types: &[TokenStream],
    header_version: (u8, u8, u8),
) -> Result<TokenStream> {
    let (major, minor, patch) = header_version;
    let header_field_ident = field_idents
        .iter()
        .map(build_header_field_ident)
        .collect::<Vec<_>>();
    Ok(quote! {
        #vis struct #header_type_ident #impl_generics #where_clause {
            #vis version: mini3d::utils::version::Version,
            #(#header_field_ident: <#field_types as mini3d::serialize::Serialize>::Header),*
        }

        impl #impl_generics #header_type_ident #ty_generics #where_clause {
            #vis fn new() -> Self {
                Self {
                    version: mini3d::utils::version::Version::new(#major, #minor, #patch),
                    #(#header_field_ident: <#field_types as mini3d::serialize::Serialize>::Header::default()),*
                }
            }
        }

        impl #impl_generics core::default::Default for #header_type_ident #ty_generics #where_clause {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #impl_generics mini3d::serialize::Serialize for #header_type_ident #ty_generics #where_clause {

            type Header = ();

            fn serialize(&self, encoder: &mut impl mini3d::serialize::Encoder) -> Result<(), mini3d::serialize::EncoderError> {
                encoder.write_u32(self.version.into())?;
                #(self.#header_field_ident.serialize(encoder)?;)*
                Ok(())
            }

            fn deserialize(decoder: &mut impl mini3d::serialize::Decoder, _header: &Self::Header) -> Result<Self, mini3d::serialize::DecoderError> {
                let version: mini3d::utils::version::Version = decoder.read_u32()?.into();
                if version != mini3d::utils::version::Version::core() {
                    return Err(mini3d::serialize::DecoderError::Unsupported);
                }
                Ok(Self {
                    version: mini3d::utils::version::Version::core(),
                    #(#header_field_ident: <#field_types as mini3d::serialize::Serialize>::Header::deserialize(decoder, &<<#field_types as mini3d::serialize::Serialize>::Header as mini3d::serialize::Serialize>::Header::default())?,)*
                })
            }
        }
    })
}

fn generate_header_struct(
    header_type_ident: &Ident,
    vis: &Visibility,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    entries: &[StructFieldEntry],
    header_version: (u8, u8, u8),
) -> Result<TokenStream> {
    let field_types = entries
        .iter()
        .filter(|entry| !entry.attributes.skip)
        .map(|entry| entry.ty.to_token_stream())
        .collect::<Vec<_>>();
    let field_idents = entries
        .iter()
        .filter(|entry| !entry.attributes.skip)
        .map(|entry| entry.ident.clone())
        .collect::<Vec<_>>();
    generate_header(
        header_type_ident,
        vis,
        impl_generics,
        ty_generics,
        where_clause,
        &field_idents,
        &field_types,
        header_version,
    )
}

fn generate_header_tuple(
    header_type_ident: &Ident,
    vis: &Visibility,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
    entries: &[TupleFieldEntry],
    header_version: (u8, u8, u8),
) -> Result<TokenStream> {
    let field_types = entries
        .iter()
        .filter(|entry| !entry.attributes.skip)
        .map(|entry| entry.ty.to_token_stream())
        .collect::<Vec<_>>();
    let field_idents = entries
        .iter()
        .filter(|entry| !entry.attributes.skip)
        .map(|entry| Ident::new(&format!("field{}", entry.index), Span::call_site()))
        .collect::<Vec<_>>();
    generate_header(
        header_type_ident,
        vis,
        impl_generics,
        ty_generics,
        where_clause,
        &field_idents,
        &field_types,
        header_version,
    )
}

fn generate_struct_field_deserialize(entry: &StructFieldEntry) -> Result<TokenStream> {
    let field_ident_header = build_header_field_ident(&entry.ident);
    let field_type = &entry.ty;
    Ok(if entry.attributes.skip {
        if let Some(expr) = &entry.attributes.default {
            quote! { #expr }
        } else {
            quote! { <#field_type as core::default::Default>::default() }
        }
    } else if let Some((major, minor, patch)) = entry.attributes.since {
        quote! {
            if header.version >= mini3d::utils::version::Version::new(#major, #minor, #patch) {
                <#field_type as mini3d::serialize::Serialize>::deserialize(decoder, &header.#field_ident_header)?
            } else {
                <#field_type as core::default::Default>::default()
            }
        }
    } else {
        quote! { <#field_type as mini3d::serialize::Serialize>::deserialize(decoder, &header.#field_ident_header)? }
    })
}

fn generate_tuple_field_deserialize(entry: &TupleFieldEntry) -> Result<TokenStream> {
    let field_ident_header = build_header_field_ident(&build_tuple_field_ident(entry.index));
    let field_type = &entry.ty;
    Ok(if entry.attributes.skip {
        if let Some(expr) = &entry.attributes.default {
            quote! { #expr }
        } else {
            quote! { <#field_type as core::default::Default>::default() }
        }
    } else if let Some((major, minor, patch)) = entry.attributes.since {
        quote! {
            if header.version >= mini3d::utils::version::Version::new(#major, #minor, #patch) {
                <#field_type as mini3d::serialize::Serialize>::deserialize(decoder, &header.#field_ident_header)?
            } else {
                <#field_type as core::default::Default>::default()
            }
        }
    } else {
        quote! { <#field_type as mini3d::serialize::Serialize>::deserialize(decoder, &header.#field_ident_header)? }
    })
}

fn build_header_type_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("{}Header", ident), Span::call_site())
}

fn build_header_field_ident(ident: &Ident) -> Ident {
    // Prevent collision with 'version' field in header
    Ident::new(&format!("{}_header", ident), Span::call_site())
}

fn build_tuple_field_ident(index: usize) -> Ident {
    Ident::new(&format!("field{}", index), Span::call_site())
}

pub(crate) fn derive_struct(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    fields: &FieldsNamed,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Parse attributes
    let struct_attributes = StructAttributes::parse(attrs)?;

    // Build entries
    let entries = parse_struct_field_entries(&struct_attributes, fields)?;
    let field_idents = entries
        .iter()
        .map(|entry| entry.ident.clone())
        .collect::<Vec<_>>();
    let field_not_skipped = entries
        .iter()
        .filter(|entry| !entry.attributes.skip)
        .map(|entry| entry.ident.clone())
        .collect::<Vec<_>>();

    // Generate header
    let header_ident = build_header_type_ident(ident);
    let header = generate_header_struct(
        &header_ident,
        vis,
        &impl_generics,
        &ty_generics,
        where_clause,
        &entries,
        struct_attributes.version,
    )?;

    // Generate deserialization
    let field_deserialize = entries
        .iter()
        .map(generate_struct_field_deserialize)
        .collect::<Result<Vec<_>>>()?;

    // Generate tokens
    Ok(quote! {

        #header

        impl #impl_generics mini3d::serialize::Serialize for #ident #ty_generics #where_clause {

            type Header = #header_ident #ty_generics;

            fn serialize(&self, encoder: &mut impl mini3d::serialize::Encoder) -> Result<(), mini3d::serialize::EncoderError> {
                #(self.#field_not_skipped.serialize(encoder)?;)*
                Ok(())
            }

            fn deserialize(decoder: &mut impl mini3d::serialize::Decoder, header: &Self::Header) -> Result<Self, mini3d::serialize::DecoderError> {
                #(let #field_idents = #field_deserialize;)*
                Ok(Self {
                    #(#field_idents),*
                })
            }
        }
    })
}

pub(crate) fn derive_tuple(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    fields: &FieldsUnnamed,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Parse attributes
    let tuple_attributes = TupleAttributes::parse(attrs)?;

    // Build entries
    let entries = parse_tuple_field_entries(&tuple_attributes, fields)?;
    let field_not_skipped = entries
        .iter()
        .filter(|entry| !entry.attributes.skip)
        .map(|entry| Index::from(entry.index))
        .collect::<Vec<_>>();

    // Generate header
    let header_ident = build_header_type_ident(ident);
    let header = generate_header_tuple(
        &header_ident,
        vis,
        &impl_generics,
        &ty_generics,
        where_clause,
        &entries,
        tuple_attributes.version,
    )?;

    // Generate deserialization
    let field_deserialize = entries
        .iter()
        .map(generate_tuple_field_deserialize)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {

        #header

        impl #impl_generics mini3d::serialize::Serialize for #ident #ty_generics #where_clause {

            type Header = #header_ident #ty_generics;

            fn serialize(&self, encoder: &mut impl mini3d::serialize::Encoder) -> Result<(), mini3d::serialize::EncoderError> {
                #(self.#field_not_skipped.serialize(encoder)?;)*
                Ok(())
            }

            fn deserialize(decoder: &mut impl mini3d::serialize::Decoder, header: &Self::Header) -> Result<Self, mini3d::serialize::DecoderError> {
                Ok(Self(
                    #(#field_deserialize),*
                ))
            }
        }
    })
}

pub(crate) fn derive_enum(
    ident: &Ident,
    vis: &Visibility,
    attrs: &[Attribute],
    generics: &Generics,
    data: &DataEnum,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Parse attributes
    let enum_attributes = EnumAttributes::parse(attrs)?;

    // Build entries
    let entries = parse_enum_field_entries(&enum_attributes, &data.variants)?;

    // Generate variant headers
    let mut variant_header_structs = Vec::new();
    let mut variant_header_fields = Vec::new();
    let mut variant_match_serialize = Vec::new();
    let mut variant_match_deserialize = Vec::new();
    for entry in &entries {
        match entry {
            EnumFieldEntry::Struct {
                ident: variant_ident,
                hash,
                entries,
            } => {
                let header_ident = build_header_type_ident(variant_ident);
                // let field_idents = entries.iter().map(|entry| build_header_field_ident(&entry.ident)).collect::<Vec<_>>();
                let field_idents = entries
                    .iter()
                    .map(|entry| entry.ident.clone())
                    .collect::<Vec<_>>();
                // let field_not_skipped = entries.iter().filter(|entry| !entry.attributes.skip).map(|entry| build_header_field_ident(&entry.ident)).collect::<Vec<_>>();
                let field_not_skipped = entries
                    .iter()
                    .filter(|entry| !entry.attributes.skip)
                    .map(|entry| entry.ident.clone())
                    .collect::<Vec<_>>();

                // Generate field
                let lower_case = &Ident::new(
                    &variant_ident.to_string().to_lowercase(),
                    variant_ident.span(),
                );
                variant_header_fields.push(lower_case.clone());
                let variant_header_field = build_header_field_ident(lower_case);

                // Generate header
                variant_header_structs.push(generate_header_struct(
                    &header_ident,
                    vis,
                    &impl_generics,
                    &ty_generics,
                    where_clause,
                    entries,
                    enum_attributes.version,
                )?);

                // Generate serialize
                variant_match_serialize.push(quote! {
                    Self::#variant_ident { #(ref #field_idents),* } => {
                        encoder.write_u32(#hash)?;
                        #(#field_not_skipped.serialize(encoder)?;)*
                    }
                });

                // Generate deserialize
                let field_deserialize = entries
                    .iter()
                    .map(generate_struct_field_deserialize)
                    .collect::<Result<Vec<_>>>()?;
                variant_match_deserialize.push(quote! {
                    #hash => {
                        let header = &header.#variant_header_field;
                        #(let #field_idents = #field_deserialize;)*
                        Ok(Self::#variant_ident {
                            #(#field_idents),*
                        })
                    }
                });
            }
            EnumFieldEntry::Tuple {
                ident: variant_ident,
                hash,
                entries,
            } => {
                let header_ident = build_header_type_ident(variant_ident);
                let field_idents = entries
                    .iter()
                    .map(|entry| build_tuple_field_ident(entry.index))
                    .collect::<Vec<_>>();
                let field_not_skipped = entries
                    .iter()
                    .filter(|entry| !entry.attributes.skip)
                    .map(|entry| build_tuple_field_ident(entry.index))
                    .collect::<Vec<_>>();
                variant_header_structs.push(generate_header_tuple(
                    &header_ident,
                    vis,
                    &impl_generics,
                    &ty_generics,
                    where_clause,
                    entries,
                    enum_attributes.version,
                )?);

                // Generate field
                let lower_case = &Ident::new(
                    &variant_ident.to_string().to_lowercase(),
                    variant_ident.span(),
                );
                variant_header_fields.push(lower_case.clone());
                let variant_header_field = build_header_field_ident(lower_case);

                // Generate serialize
                variant_match_serialize.push(quote! {
                    Self::#variant_ident(#(ref #field_idents),*) => {
                        encoder.write_u32(#hash)?;
                        #(#field_not_skipped.serialize(encoder)?;)*
                    }
                });

                // Generate deserialize
                let field_deserialize = entries
                    .iter()
                    .map(generate_tuple_field_deserialize)
                    .collect::<Result<Vec<_>>>()?;
                variant_match_deserialize.push(quote! {
                    #hash => {
                        let header = &header.#variant_header_field;
                        #(let #field_idents = #field_deserialize;)*
                        Ok(Self::#variant_ident(
                            #(#field_idents),*
                        ))
                    }
                });
            }
            EnumFieldEntry::Unit {
                ident: variant_ident,
                hash,
            } => {
                let header_ident = build_header_type_ident(variant_ident);
                variant_header_structs.push(generate_header(
                    &header_ident,
                    vis,
                    &impl_generics,
                    &ty_generics,
                    where_clause,
                    &Vec::new(),
                    &Vec::new(),
                    enum_attributes.version,
                )?);

                // Generate field
                let lower_case = &Ident::new(
                    &variant_ident.to_string().to_lowercase(),
                    variant_ident.span(),
                );
                variant_header_fields.push(lower_case.clone());

                // Generate serialize
                variant_match_serialize.push(quote! {
                    Self::#variant_ident => {
                        encoder.write_u32(#hash)?;
                    }
                });

                // Generate deserialize
                variant_match_deserialize.push(quote! {
                    #hash => {
                        Ok(Self::#variant_ident)
                    }
                });
            }
        }
    }

    // Generate enum header
    let header_ident = build_header_type_ident(ident);
    let mut header_types = Vec::new();
    for entry in entries {
        let ident = match entry {
            EnumFieldEntry::Struct { ident, .. } => ident,
            EnumFieldEntry::Tuple { ident, .. } => ident,
            EnumFieldEntry::Unit { ident, .. } => ident,
        };
        header_types.push(build_header_type_ident(&ident).to_token_stream());
    }

    let (major, minor, patch) = enum_attributes.version;
    let variant_header_fields = variant_header_fields
        .iter()
        .map(build_header_field_ident)
        .collect::<Vec<_>>();
    let header = quote! {
        #vis struct #header_ident #impl_generics #where_clause {
            version: mini3d::utils::version::Version,
            #(#variant_header_fields: #header_types),*
        }

        impl #impl_generics #header_ident #ty_generics #where_clause {
            fn new() -> Self {
                Self {
                    version: mini3d::utils::version::Version::new(#major, #minor, #patch),
                    #(#variant_header_fields: #header_types::default()),*
                }
            }
        }

        impl #impl_generics core::default::Default for #header_ident #ty_generics #where_clause {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #impl_generics mini3d::serialize::Serialize for #header_ident #ty_generics #where_clause {

            type Header = ();

            fn serialize(&self, encoder: &mut impl mini3d::serialize::Encoder) -> Result<(), mini3d::serialize::EncoderError> {
                encoder.write_u32(self.version.into())?;
                #(self.#variant_header_fields.serialize(encoder)?;)*
                Ok(())
            }

            fn deserialize(decoder: &mut impl mini3d::serialize::Decoder, _header: &Self::Header) -> Result<Self, mini3d::serialize::DecoderError> {
                let version: mini3d::utils::version::Version = decoder.read_u32()?.into();
                if version != mini3d::utils::version::Version::core() {
                    return Err(mini3d::serialize::DecoderError::Unsupported);
                }
                Ok(Self {
                    version: mini3d::utils::version::Version::core(),
                    #(#variant_header_fields: <#header_types as mini3d::serialize::Serialize>::deserialize(decoder, &<#header_types as mini3d::serialize::Serialize>::Header::default())?,)*
                })
            }
        }
    };

    Ok(quote! {

        #(#variant_header_structs)*
        #header

        impl #impl_generics mini3d::serialize::Serialize for #ident #ty_generics #where_clause {

            type Header = #header_ident #ty_generics;

            fn serialize(&self, encoder: &mut impl mini3d::serialize::Encoder) -> Result<(), mini3d::serialize::EncoderError> {
                match self {
                    #(#variant_match_serialize,)*
                }
                Ok(())
            }

            fn deserialize(decoder: &mut impl mini3d::serialize::Decoder, header: &Self::Header) -> Result<Self, mini3d::serialize::DecoderError> {
                let hash = decoder.read_u32()?;
                match hash {
                    #(#variant_match_deserialize,)*
                    _ => Err(mini3d::serialize::DecoderError::CorruptedData),
                }
            }
        }
    })
}
