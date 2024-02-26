use mini3d_asset::{font::Font, material::Material, mesh::Mesh, texture::Texture};
use mini3d_utils::string::AsciiArray;

pub struct AssetImportEntry<T> {
    pub name: AsciiArray<32>,
    pub data: T,
}

pub enum ImportAssetEvent {
    Font(AssetImportEntry<Font>),
    Material(AssetImportEntry<Material>),
    Mesh(AssetImportEntry<Mesh>),
    Texture(AssetImportEntry<Texture>),
}
