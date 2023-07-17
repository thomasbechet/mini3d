use std::collections::HashMap;

use crate::{
    ecs::system::{ExclusiveSystemCallback, ParallelSystemCallback},
    feature::component::common::program::Program,
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::{
    component::{ComponentId, ComponentRegistry},
    error::RegistryError,
};

pub(crate) type SystemId = SlotId<SystemDefinition>;

#[derive(Clone)]
pub(crate) enum ExclusiveSystem {
    Callback(ExclusiveSystemCallback),
    Program(Program),
}

#[derive(Clone)]
pub(crate) enum ParallelSystem {
    Callback(ParallelSystemCallback),
    Program(Program),
}

#[derive(Clone)]
pub(crate) enum ParallelForSytem {
    Callback(ParallelSystemCallback),
    Program(Program),
}

#[derive(Clone)]
pub(crate) enum System {
    Exclusive(ExclusiveSystem),
    Parallel(ParallelSystem),
    ParallelFor(ParallelForSytem),
}

pub struct ExclusiveComponentResolver<'a> {
    registry: &'a ComponentRegistry,
}

impl<'a> ExclusiveComponentResolver<'a> {
    pub fn find(&mut self, component: UID) -> Result<ComponentId, RegistryError> {}
}

pub struct ParallelComponentResolver<'a> {
    registry: &'a ComponentRegistry,
    components: Vec<ComponentId>,
}

impl<'a> ParallelComponentResolver<'a> {
    pub fn find(&mut self, component: UID) -> Result<ComponentId, RegistryError> {}
}

pub trait ExclusiveSystem: 'static {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve(&mut self, resolver: &ExclusiveComponentResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult;
}

pub trait ParallelSystem: 'static {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve(&mut self, resolver: &mut ParallelComponentResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ParallelContext) -> SystemResult;
}

pub(crate) struct SystemDefinition {
    pub(crate) name: String,
    pub(crate) system: System,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: SlotMap<SystemDefinition>,
    lookup_cache: HashMap<UID, SystemId>,
}

impl SystemRegistry {
    fn define(&mut self, definition: SystemDefinition) -> Result<SystemId, RegistryError> {
        let uid: UID = definition.name.as_str().into();
        if self.find(uid).is_some() {
            return Err(RegistryError::DuplicatedSystemDefinition {
                name: definition.name,
            });
        }
        let id = self.systems.add(definition);
        self.lookup_cache.insert(uid, id);
        Ok(id)
    }

    pub(crate) fn define_exclusive_callback(
        &mut self,
        name: &str,
        callback: ExclusiveSystemCallback,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            system: System::Exclusive(ExclusiveSystem::Callback(callback)),
        })
    }

    pub(crate) fn define_parallel_callback(
        &mut self,
        name: &str,
        callback: ParallelSystemCallback,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            system: System::Parallel(ParallelSystem::Callback(callback)),
        })
    }

    pub(crate) fn define_exclusive_program(
        &mut self,
        name: &str,
        program: Program,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            system: System::Exclusive(ExclusiveSystem::Program(program)),
        })
    }

    pub(crate) fn find(&self, uid: UID) -> Option<SystemId> {
        self.lookup_cache.get(&uid).copied()
    }

    pub(crate) fn get(&self, id: SystemId) -> Option<&SystemDefinition> {
        self.systems.get(id)
    }
}
