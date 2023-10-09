use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Duplicated component")]
    DuplicatedComponent,
    #[error("Duplicated resource")]
    DuplicatedResource,
    #[error("Duplicated system")]
    DuplicatedSystem,
    #[error("Component not found")]
    ComponentNotFound,
    #[error("System not found")]
    SystemNotFound,
}
