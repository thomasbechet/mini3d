use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum ECSError {
    #[error("Singleton type mismatch")]
    ContainerBorrowMut,
    #[error("System stage not found")]
    SystemStageNotFound,
}
