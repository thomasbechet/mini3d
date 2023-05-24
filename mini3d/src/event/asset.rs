use mini3d_derive::Serialize;

use crate::{feature::asset::{font::Font, mesh::Mesh, material::Material, model::Model, texture::Texture, script::Script}, registry::asset::Asset};

#[derive(Serialize)]
pub struct AssetImportEntry<A: Asset> {
    pub name: String,
    pub data: A,
}

#[derive(Serialize)]
pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Material>),
    Mesh(AssetImportEntry<Mesh>),
    Model(AssetImportEntry<Model>),
    Script(AssetImportEntry<Script>),
    Texture(AssetImportEntry<Texture>),
}