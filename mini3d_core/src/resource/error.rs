use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Duplicated resource entry")]
    DuplicatedAssetEntry,
    #[error("Resource not found")]
    ResourceNotFound,
    #[error("Resource not loaded")]
    ResourceNotLoaded,
    #[error("Resource type not found")]
    ResourceTypeNotFound,
    #[error("Deserialization error")]
    DeserializationError,
    #[error("Serialization error")]
    SerializationError,
}
