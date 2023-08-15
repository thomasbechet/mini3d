use mini3d_derive::Error;

use crate::utils::uid::UID;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Duplicated asset entry")]
    DuplicatedAssetEntry,
    #[error("Duplicated asset type: {uid}")]
    DuplicatedAssetType { uid: UID },
    #[error("Invalid asset type cast")]
    InvalidAssetTypeCast,
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
