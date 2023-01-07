use serde::{Serialize, Deserialize};

use crate::feature::asset::{font::FontAsset, mesh::MeshAsset, material::MaterialAsset, model::ModelAsset, rhai_script::RhaiScriptAsset, texture::TextureAsset};

#[derive(Serialize, Deserialize)]
pub struct AssetImportEntry<T> {
    pub name: String,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub enum ImportAssetEvent {
    Font(AssetImportEntry<FontAsset>),
    Material(AssetImportEntry<MaterialAsset>),
    Mesh(AssetImportEntry<MeshAsset>),
    Model(AssetImportEntry<ModelAsset>),
    RhaiScript(AssetImportEntry<RhaiScriptAsset>),
    Texture(AssetImportEntry<TextureAsset>),
}