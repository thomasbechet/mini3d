use crate::asset::{font::Font, material::Material, mesh::Mesh, texture::Texture, Asset, AssetName};

pub struct AssetImport<T: Asset> {
    pub data: Box<T>,
    pub name: AssetName,
}

pub enum ImportAssetEvent {
    Font(AssetImport<Font>),
    Material(AssetImport<Material>),
    Mesh(AssetImport<Mesh>),
    Texture(AssetImport<Texture>),
}

pub enum AssetEvent {
    Import(ImportAssetEvent),
}