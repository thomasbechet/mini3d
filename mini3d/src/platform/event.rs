use mini3d_derive::Serialize;

use crate::{resource::handle::MAX_RESOURCE_NAME_LEN, utils::string::AsciiArray};

pub struct AssetImportEntry<T> {
    pub name: AsciiArray<MAX_RESOURCE_NAME_LEN>,
    pub data: T,
}

pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Materiat>),
    Mesh(AssetImportEntry<Mesh>),
    Model(AssetImportEntry<Model>),
    Script(AssetImportEntry<Script>),
    Texture(AssetImportEntry<Texture>),
}

#[derive(Serialize)]
pub enum PlatformEvent {
    RequestStop,
}
