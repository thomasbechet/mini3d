use mini3d_derive::Serialize;

use crate::{
    feature::{
        common::script::Script,
        renderer::{font::Font, material::Material, mesh::Mesh, model::Model, texture::Texture},
    },
    registry::asset::AssetData,
};

#[derive(Serialize)]
pub struct AssetImportEntry<A: AssetData> {
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

#[derive(Serialize)]
pub enum SystemEvent {
    RequestStop,
}
