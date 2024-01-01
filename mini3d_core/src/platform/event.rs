use mini3d_derive::Serialize;

use crate::{
    renderer::resource::{Font, Material, Mesh, Model, Texture},
    script::component::Script,
    utils::string::AsciiArray,
};

pub struct AssetImportEntry<T> {
    pub name: AsciiArray<32>,
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
