use mini3d_derive::Serialize;

use crate::{
    feature::component::{
        common::script::Script,
        renderer::{font::Font, material::Material, mesh::Mesh, model::Model, texture::Texture},
    },
    registry::component::Component,
};

#[derive(Serialize)]
pub struct AssetImportEntry<C: Component> {
    pub name: String,
    pub data: C,
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
