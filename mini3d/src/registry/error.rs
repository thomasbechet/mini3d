use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Duplicated asset definition: {name}")]
    DuplicatedAssetDefinition { name: String },
    #[error("Duplicated component definition: {name}")]
    DuplicatedComponentDefinition { name: String },
    #[error("Duplicated system definition: {name}")]
    DuplicatedSystemDefinition { name: String },
    #[error("Asset definition not found")]
    AssetDefinitionNotFound,
    #[error("Component definition not found")]
    ComponentDefinitionNotFound,
    #[error("System definition not found")]
    SystemDefinitionNotFound,
    #[error("Incompatible system stage definition")]
    IncompatibleSystemStageDefinition,
}
