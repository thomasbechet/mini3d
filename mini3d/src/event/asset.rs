use crate::asset::{font::Font, material::Material, mesh::Mesh, texture::Texture, Asset, rhai_script::RhaiScript, model::Model};

pub struct AssetImport<T: Asset> {
    pub data: T,
    pub name: String,
}

pub enum ImportAssetEvent {
    Font(AssetImport<Font>),
    Material(AssetImport<Material>),
    Mesh(AssetImport<Mesh>),
    Model(AssetImport<Model>),
    RhaiScript(AssetImport<RhaiScript>),
    Texture(AssetImport<Texture>),
}