use mini3d_derive::Error;

use crate::registry::error::RegistryError;

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Registry error: {0}")]
    Registry(RegistryError),
    #[error("System error")]
    SystemError,
    #[error("Component type mismatch")]
    ComponentTypeMismatch,
    #[error("Singleton type mismatch")]
    ContainerBorrowMut,
    #[error("System already exists")]
    SystemAlreadyExists,
    #[error("System stage already exists")]
    SystemStageAlreadyExists,
    #[error("System stage not found")]
    SystemStageNotFound,
    #[error("System not found")]
    SystemNotFound,
}
