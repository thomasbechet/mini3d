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
