use mini3d_derive::Error;

use crate::utils::uid::UID;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Duplicated asset definition: {name}")]
    DuplicatedAssetDefinition { name: String },
    #[error("Duplicated component definition: {name}")]
    DuplicatedComponentDefinition { name: String },
    #[error("Duplicated system definition: {name}")]
    DuplicatedSystemDefinition { name: String },
    #[error("Asset definition not found: {uid}")]
    AssetDefinitionNotFound { uid: UID },
    #[error("Component definition not found: {uid}")]
    ComponentDefinitionNotFound { uid: UID },
    #[error("System definition not found: {uid}")]
    SystemDefinitionNotFound { uid: UID },
}
