use serde::{Serialize, Deserialize};

use crate::feature::asset::{font::Font, mesh::Mesh, material::Material, model::Model, texture::Texture, script::Script};

#[derive(Serialize, Deserialize)]
pub struct AssetImportEntry<T> {
    pub name: String,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Material>),
    Mesh(AssetImportEntry<Mesh>),
    Model(AssetImportEntry<Model>),
    Script(AssetImportEntry<Script>),
    Texture(AssetImportEntry<Texture>),
}