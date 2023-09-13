use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Duplicated asset entry")]
    DuplicatedAssetEntry,
    #[error("Asset not found")]
    AssetNotFound,
    #[error("Asset type not found")]
    AssetTypeNotFound,
    #[error("Bundle not found")]
    BundleNotFound,
    #[error("Duplicated bundle")]
    DuplicatedBundle,
    #[error("Deserialization error")]
    DeserializationError,
    #[error("Serialization error")]
    SerializationError,
}
