use serde::{Serialize, Deserialize};

use crate::asset::{font::Font, material::Material, mesh::Mesh, texture::Texture, rhai_script::RhaiScript, model::Model};

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
    RhaiScript(AssetImportEntry<RhaiScript>),
    Texture(AssetImportEntry<Texture>),
}