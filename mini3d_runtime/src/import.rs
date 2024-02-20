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
