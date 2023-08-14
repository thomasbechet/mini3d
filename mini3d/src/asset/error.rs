use mini3d_derive::Error;

use crate::utils::uid::UID;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Duplicated asset entry: {name}")]
    DuplicatedAssetEntry { name: String },
    #[error("Duplicated asset type: {uid}")]
    DuplicatedAssetType { uid: UID },
    #[error("Invalid asset type cast")]
    InvalidAssetTypeCast,
    #[error("Asset not found")]
    AssetNotFound,
    #[error("Asset type not found")]
    AssetTypeNotFound,
    #[error("Bundle not found: {uid}")]
    BundleNotFound { uid: UID },
    #[error("Duplicated bundle entry: {name}")]
    DuplicatedBundleEntry { name: String },
    #[error("Deserialization error")]
    DeserializationError,
    #[error("Serialization error")]
    SerializationError,
}
