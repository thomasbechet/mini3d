use std::{fmt::Display, error::Error};

use crate::uid::UID;

#[derive(Debug)]
pub enum RegistryError {
    DuplicatedAssetDefinition { name: String },
    DuplicatedComponentDefinition { name: String },
    DuplicatedSystemDefinition { name: String },
    AssetDefinitionNotFound { uid: UID },
    ComponentDefinitionNotFound { uid: UID },
    SystemDefinitionNotFound { uid: UID },
}

impl Error for RegistryError {}

impl Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::DuplicatedAssetDefinition { name } => write!(f, "Duplicated asset definition: {}", name),
            RegistryError::DuplicatedComponentDefinition { name } => write!(f, "Duplicated component definition: {}", name),
            RegistryError::DuplicatedSystemDefinition { name } => write!(f, "Duplicated system definition {}", name),
            RegistryError::AssetDefinitionNotFound { uid } => write!(f, "Asset definition not found: {}", uid),
            RegistryError::ComponentDefinitionNotFound { uid } => write!(f, "Component definition not found: {}", uid),
            RegistryError::SystemDefinitionNotFound { uid } => write!(f, "System definition not found: {}", uid),
        }
    }
}