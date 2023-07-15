use std::collections::HashMap;

use crate::{
    ecs::system::{ExclusiveSystemCallback, ParallelSystemCallback},
    uid::UID,
};

use super::error::RegistryError;

#[derive(Clone, Copy)]
pub(crate) enum ExclusiveSystem {
    Callback(ExclusiveSystemCallback),
    Module(UID),
}

#[derive(Clone, Copy)]
pub(crate) enum ParallelSystem {
    Callback(ParallelSystemCallback),
    Module(UID),
}

#[derive(Clone, Copy)]
pub(crate) enum SystemKind {
    Exclusive(ExclusiveSystem),
    Parallel(ParallelSystem),
}

pub(crate) struct SystemDefinition {
    pub(crate) name: String,
    pub(crate) kind: SystemKind,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: HashMap<UID, SystemDefinition>,
}

impl SystemRegistry {
    fn define(&mut self, definition: SystemDefinition) -> Result<UID, RegistryError> {
        let uid: UID = definition.name.as_str().into();
        if self.systems.contains_key(&uid) {
            return Err(RegistryError::DuplicatedSystemDefinition {
                name: definition.name,
            });
        }
        self.systems.insert(uid, definition);
        Ok(uid)
    }

    pub(crate) fn define_static_exclusive(
        &mut self,
        name: &str,
        callback: ExclusiveSystemCallback,
    ) -> Result<UID, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            kind: SystemKind::Exclusive(ExclusiveSystem::Callback(callback)),
        })
    }

    pub(crate) fn define_static_parallel(
        &mut self,
        name: &str,
        callback: ParallelSystemCallback,
    ) -> Result<UID, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            kind: SystemKind::Parallel(ParallelSystem::Callback(callback)),
        })
    }

    pub(crate) fn define_script_exclusive()

    pub(crate) fn get(&self, uid: &UID) -> Option<&SystemDefinition> {
        self.systems.get(uid)
    }
}
