use mini3d_renderer::{font::Font, material::Material, mesh::MeshData, texture::TextureData};
use mini3d_utils::string::AsciiArray;

pub struct AssetImportEntry<T> {
    pub name: AsciiArray<32>,
    pub data: T,
}

pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Material>),
    Mesh(AssetImportEntry<MeshData>),
    Texture(AssetImportEntry<TextureData>),
}
