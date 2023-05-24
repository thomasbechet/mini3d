use mini3d_derive::Error;

use crate::{uid::UID, registry::error::RegistryError};

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
pub enum WorldError {
    #[error("Registry error: {0}")]
    Registry(RegistryError),
    #[error("Duplicated world: {name}")]
    DuplicatedWorld { name: String },
    #[error("World not found: {uid}")]
    WorldNotFound { uid: UID },
    #[error("Change to removed world: {uid}")]
    ChangeToRemovedWorld { uid: UID },
    #[error("Remove and change same world: {uid}")]
    RemoveAndChangeSameWorld { uid: UID },
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