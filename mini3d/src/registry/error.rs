use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Duplicated component: {name}")]
    DuplicatedComponent { name: String },
    #[error("Duplicated system: {name}")]
    DuplicatedSystem { name: String },
    #[error("Component not found")]
    ComponentNotFound,
    #[error("System not found")]
    SystemNotFound,
    #[error("System stage not found")]
    SystemStageNotFound,
}
