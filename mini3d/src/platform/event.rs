use mini3d_derive::Serialize;

use crate::feature::{
    common::script::Script,
    renderer::{font::Font, material::Material, mesh::Mesh, texture::Texture},
};

#[derive(Serialize)]
pub struct AssetImportEntry<T> {
    pub name: String,
    pub data: T,
}

#[derive(Serialize)]
pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Material>),
    Mesh(AssetImportEntry<Mesh>),
    // Model(AssetImportEntry<Model>),
    Script(AssetImportEntry<Script>),
    Texture(AssetImportEntry<Texture>),
}

#[derive(Serialize)]
pub enum PlatformEvent {
    RequestStop,
}
