use std::{error::Error, fmt::Display};

use crate::{uid::UID, registry::error::RegistryError};

#[derive(Debug)]
pub enum ECSError {
    SystemNotFound { uid: UID },
    ContainerBorrowMut,
    SystemError,
    RegistryError,
}

impl Error for ECSError {}

impl Display for ECSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ECSError::SystemNotFound { uid } => write!(f, "System not found: {}", uid),
            ECSError::ContainerBorrowMut => write!(f, "Container already borrowed mutably"),
            ECSError::SystemError => write!(f, "System error"),
            ECSError::RegistryError => write!(f, "Registry error"),
        }
    }
}

#[derive(Debug)]
pub enum SchedulerError {
    GroupNotFound { uid: UID },
    DuplicatedGroup { name: String },
}

impl Error for SchedulerError {}

impl Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerError::GroupNotFound { uid } => write!(f, "Group not found: {}", uid),
            SchedulerError::DuplicatedGroup { name } => write!(f, "Duplicated group: {}", name),
        }
    }
}

#[derive(Debug)]
pub enum WorldError {
    Registry(RegistryError),
    DuplicatedWorld { name: String },
    WorldNotFound { uid: UID },
    ChangeToRemovedWorld { uid: UID },
    RemoveAndChangeSameWorld { uid: UID },
    ComponentContainerNotFound { uid: UID },
    ComponentTypeMismatch { uid: UID },
    SingletonTypeMismatch { uid: UID },
    SingletonNotFound { uid: UID },
    DuplicatedSingleton { uid: UID },
    ContainerBorrowMut,
}

impl Error for WorldError {}

impl Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldError::Registry(err) => write!(f, "Registry error: {}", err),
            WorldError::DuplicatedWorld { name } => write!(f, "Duplicated world: {}", name),
            WorldError::WorldNotFound { uid } => write!(f, "World not found: {}", uid),
            WorldError::ChangeToRemovedWorld { uid } => write!(f, "Change to removed world: {}", uid),
            WorldError::RemoveAndChangeSameWorld { uid } => write!(f, "Remove and change same world: {}", uid),
            WorldError::ComponentContainerNotFound { uid } => write!(f, "Component container not found: {}", uid),
            WorldError::ComponentTypeMismatch { uid } => write!(f, "Component type mismatch: {}", uid),
            WorldError::SingletonTypeMismatch { uid } => write!(f, "Singleton type mismatch: {}", uid),
            WorldError::SingletonNotFound { uid } => write!(f, "Singleton not found: {}", uid),
            WorldError::DuplicatedSingleton { uid } => write!(f, "Duplicated singleton: {}", uid),
            WorldError::ContainerBorrowMut => write!(f, "Container already borrowed mutably"),
        }
    }
}