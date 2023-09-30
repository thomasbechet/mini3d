use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Duplicated asset entry")]
    DuplicatedAssetEntry,
    #[error("Asset not found")]
    AssetNotFound,
    #[error("Asset not loaded")]
    AssetNotLoaded,
    #[error("Asset type not found")]
    AssetTypeNotFound,
    #[error("Deserialization error")]
    DeserializationError,
    #[error("Serialization error")]
    SerializationError,
}
