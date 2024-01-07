use mini3d_derive::Serialize;

use crate::{
    renderer::component::{FontData, MaterialData, MeshData, TextureData},
    script::component::Script,
    utils::string::AsciiArray,
};

pub struct AssetImportEntry<T> {
    pub name: AsciiArray<32>,
    pub data: T,
}

pub enum ImportAssetEvent {
    Font(AssetImportEntry<FontData>),
    Material(AssetImportEntry<MaterialData>),
    Mesh(AssetImportEntry<MeshData>),
    Model(AssetImportEntry<Model>),
    Script(AssetImportEntry<Script>),
    Texture(AssetImportEntry<TextureData>),
}

#[derive(Serialize)]
pub enum PlatformEvent {
    RequestStop,
}
