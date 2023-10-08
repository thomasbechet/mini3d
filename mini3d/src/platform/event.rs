use mini3d_derive::Serialize;

use crate::{
    feature::{
        common::script::Script,
        renderer::{font::Font, material::Material, mesh::Mesh, model::Model, texture::Texture},
    },
    registry::datatype::StaticDataType,
};

#[derive(Serialize)]
pub struct AssetImportEntry<C: Component> {
    pub name: String,
    pub data: D,
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
pub enum PlatformEvent {
    RequestStop,
}
