use serde::{Serialize, Deserialize};

use crate::asset::{font::Font, material::Material, mesh::Mesh, texture::Texture, Asset, rhai_script::RhaiScript, model::Model};

#[derive(Serialize, Deserialize)]
pub struct AssetImport<T: Asset> {
    pub name: String,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub enum ImportAssetEvent {
    Font(AssetImport<Font>),
    Material(AssetImport<Material>),
    Mesh(AssetImport<Mesh>),
    Model(AssetImport<Model>),
    RhaiScript(AssetImport<RhaiScript>),
    Texture(AssetImport<Texture>),
}