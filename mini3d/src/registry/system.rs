use std::collections::HashMap;

use crate::{
    ecs::{
        context::{ExclusiveContext, ParallelContext},
        system::SystemResult,
    },
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SystemId(SlotId);

impl From<SlotId> for SystemId {
    fn from(id: SlotId) -> Self {
        Self(id)
    }
}

impl From<SystemId> for SlotId {
    fn from(id: SystemId) -> Self {
        id.0
    }
}

pub struct ExclusiveResolver<'a> {
    registry: &'a ComponentRegistry,
}

impl<'a> ExclusiveResolver<'a> {
    pub fn find(&mut self, component: UID) -> Result<ComponentId, RegistryError> {
        self.registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })
    }
}

pub struct ParallelResolver<'a> {
    registry: &'a ComponentRegistry,
    reads: Vec<ComponentId>,
    writes: Vec<ComponentId>,
}

impl<'a> ParallelResolver<'a> {
    pub fn read(&mut self, component: UID) -> Result<ComponentId, RegistryError> {
        let id = self
            .registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(id)
    }
    pub fn write(&mut self, component: UID) -> Result<ComponentId, RegistryError> {
        let id = self
            .registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        if self.reads.contains(&id) {
            self.reads.retain(|&x| x != id);
        }
        if !self.writes.contains(&id) {
            self.writes.push(id);
        }
        Ok(id)
    }
}

pub trait ExclusiveSystem: 'static + Default {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve(&mut self, resolver: &ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult;
}

pub trait ParallelSystem: 'static + Default {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ParallelContext) -> SystemResult;
}

struct StaticExclusiveSystemInstance<S: ExclusiveSystem> {
    instance: S,
}

trait AnyStaticExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult;
}

impl<S: ExclusiveSystem> AnyStaticExclusiveSystemInstance for StaticExclusiveSystemInstance<S> {
    fn resolve(&mut self, resolver: &ExclusiveResolver) -> Result<(), RegistryError> {
        self.instance.resolve(resolver)
    }
    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        self.instance.run(ctx)
    }
}

struct StaticParallelSystemInstance<S: ParallelSystem> {
    instance: S,
}

trait AnyStaticParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ParallelContext) -> SystemResult;
}

impl<S: ParallelSystem> AnyStaticParallelSystemInstance for StaticParallelSystemInstance<S> {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.instance.resolve(resolver)
    }
    fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
        self.instance.run(ctx)
    }
}

enum StaticSystem {
    Exclusive(Box<dyn AnyStaticExclusiveSystemInstance>),
    Parallel(Box<dyn AnyStaticParallelSystemInstance>),
}

struct ProgramSystem {
    program: Program,
}

enum SystemInstance {
    Static(StaticSystem),
    Program(ProgramSystem),
}

pub(crate) struct SystemDefinition {
    pub(crate) name: String,
    pub(crate) instance: SystemInstance,
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
        self.lookup_cache.insert(uid, id.into());
        Ok(id.into())
    }

    fn resolve_components(&mut self) -> Result<(), RegistryError> {
        // TODO: Resolve components
        Ok(())
    }

    pub(crate) fn define_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            instance: SystemInstance::Static(StaticSystem::Exclusive(Box::new(
                StaticExclusiveSystemInstance {
                    instance: S::default(),
                },
            ))),
        })
    }

    pub(crate) fn define_static_parallel<S: ParallelSystem>(
        &mut self,
        name: &str,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.to_string(),
            instance: SystemInstance::Static(StaticSystem::Parallel(Box::new(
                StaticParallelSystemInstance {
                    instance: S::default(),
                },
            ))),
        })
    }

    pub(crate) fn find(&self, uid: UID) -> Option<SystemId> {
        self.lookup_cache.get(&uid).copied()
    }

    pub(crate) fn get(&self, id: SystemId) -> Option<&SystemDefinition> {
        self.systems.get(id.into())
    }
}
