use crate::asset::{font::Font, material::Material, mesh::Mesh, texture::Texture, Asset, script::RhaiScript};

pub struct AssetImport<T: Asset> {
    pub data: T,
    pub name: String,
}

pub enum ImportAssetEvent {
    Font(AssetImport<Font>),
    Material(AssetImport<Material>),
    Mesh(AssetImport<Mesh>),
    RhaiScript(AssetImport<RhaiScript>),
    Texture(AssetImport<Texture>),
}