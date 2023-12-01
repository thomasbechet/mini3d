use mini3d_derive::Serialize;

use crate::{
    feature::{
        common::script::Script,
        renderer::{font::Font, material::Material, mesh::Mesh, model::Model, texture::Texture},
    },
    resource::handle::MAX_RESOURCE_NAME_LEN,
    utils::string::AsciiArray,
};

pub struct AssetImportEntry<T> {
    pub name: AsciiArray<MAX_RESOURCE_NAME_LEN>,
    pub data: T,
}

pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Material>),
    Mesh(AssetImportEntry<Mesh>),
    Model(AssetImportEntry<Model>),
    Script(AssetImportEntry<Script>),
    Texture(AssetImportEntry<Texture>),
}

#[derive(Serialize)]
pub enum PlatformEvent {
    RequestStop,
}
