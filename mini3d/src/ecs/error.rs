use mini3d_derive::Error;

use crate::{registry::error::RegistryError, uid::UID};

#[derive(Debug, Error)]
pub enum ECSError {
    #[error("System not found: {uid}")]
    SystemNotFound { uid: UID },
    #[error("Container already borrowed mutably")]
    ContainerBorrowMut,
    #[error("System error")]
    SystemError,
    #[error("Registry error")]
    RegistryError,
}

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Group not found: {uid}")]
    GroupNotFound { uid: UID },
    #[error("Duplicated group: {name}")]
    DuplicatedGroup { name: String },
}

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("Registry error: {0}")]
    Registry(RegistryError),
    #[error("Duplicated scene: {name}")]
    DuplicatedScene { name: String },
    #[error("Scene not found: {uid}")]
    SceneNotFound { uid: UID },
    #[error("Change to removed scene: {uid}")]
    ChangeToRemovedScene { uid: UID },
    #[error("Remove and change same scene: {uid}")]
    RemoveAndChangeSameScene { uid: UID },
    #[error("Component container not found: {uid}")]
    ComponentContainerNotFound { uid: UID },
    #[error("Component type mismatch: {uid}")]
    ComponentTypeMismatch { uid: UID },
    #[error("Singleton type mismatch: {uid}")]
    SingletonTypeMismatch { uid: UID },
    #[error("Singleton not found: {uid}")]
    SingletonNotFound { uid: UID },
    #[error("Duplicated singleton: {uid}")]
    DuplicatedSingleton { uid: UID },
    #[error("Container already borrowed mutably")]
    ContainerBorrowMut,
}
