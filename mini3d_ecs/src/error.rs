use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum ECSError {
    #[error("Container borrow mut")]
    ContainerBorrowMut,
    #[error("Duplicated component type")]
    DuplicatedComponentType,
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Component not found")]
    ComponentNotFound,
}

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("Invalid container type")]
    InvalidContainerType,
    #[error("Component did not match unicity constraint")]
    DuplicatedEntry,
    #[error("Component reference not found")]
    UnresolvedReference,
    #[error("Component provider error")]
    ProviderError,
    #[error("Component not found")]
    EntryNotFound,
}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("System error")]
    RunError,
    #[error("System config error")]
    ConfigError,
    #[error("Duplicated entry")]
    DuplicatedEntry,
}
