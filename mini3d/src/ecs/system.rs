use crate::{
    feature::core::component_type::ComponentId,
    resource::{handle::ResourceRef, ResourceManager},
    utils::slotmap::{SlotId, SlotMap},
};

use super::{api::context::Context, container::ContainerTable, error::ResolverError};

pub(crate) struct SystemId(pub(crate) SlotId);

pub trait ExclusiveSystem: 'static + Default {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(&self, ctx: &mut Context) {}
}

pub trait ParallelSystem: 'static + Default {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), ResolverError> {
        Ok(())
    }
    fn run(&self, ctx: &Context) {}
}

use crate::{feature::common::program::Program, utils::uid::ToUID};

use super::{
    entity::EntityTable,
    query::{QueryBuilder, QueryTable},
};

pub struct ExclusiveResolver<'a> {
    system: SystemId,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    containers: &'a mut ContainerTable,
    resources: &'a mut ResourceManager,
}

impl<'a> ExclusiveResolver<'a> {
    pub fn find(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let handle = self
            .resources
            .find(component)
            .ok_or(ResolverError::ComponentNotFound)?;
        Ok(self.containers.preallocate(handle, self.resources))
    }

    pub fn query(&mut self) -> QueryBuilder<'_> {
        self.all.clear();
        self.any.clear();
        self.not.clear();
        QueryBuilder {
            system: self.system,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            queries: self.queries,
            containers: self.containers,
            resources: self.resources,
        }
    }
}

pub struct ParallelResolver<'a> {
    system: SystemId,
    reads: Vec<ComponentId>,
    writes: Vec<ComponentId>,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    containers: &'a mut ContainerTable,
    resources: &'a mut ResourceManager,
}

impl<'a> ParallelResolver<'a> {
    fn find(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let handle = self
            .resources
            .find(component)
            .ok_or(ResolverError::ComponentNotFound)?;
        Ok(self.containers.preallocate(handle, self.resources))
    }

    pub fn read(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let id = self.find(component)?;
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(id)
    }

    pub fn write(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let id = self.find(component)?;
        if self.reads.contains(&id) {
            self.reads.retain(|&x| x != id);
        }
        if !self.writes.contains(&id) {
            self.writes.push(id);
        }
        Ok(id)
    }

    pub fn query(&mut self) -> QueryBuilder<'_> {
        self.all.clear();
        self.any.clear();
        self.not.clear();
        QueryBuilder {
            system: self.system,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            queries: self.queries,
            containers: self.containers,
            resources: self.resources,
        }
    }
}

pub(crate) trait AnyNativeExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), ResolverError>;
    fn run(&self, ctx: &mut Context);
}

pub(crate) trait AnyNativeParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), ResolverError>;
    fn run(&self, ctx: &Context);
}

pub(crate) enum ExclusiveSystemInstance {
    Native(Box<dyn AnyNativeExclusiveSystemInstance>),
    Program(Program),
}

impl ExclusiveSystemInstance {
    pub(crate) fn resolve(
        &mut self,
        resolver: &mut ExclusiveResolver,
    ) -> Result<(), ResolverError> {
        match self {
            Self::Native(instance) => instance.resolve(resolver),
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run(&self, ctx: &mut Context) {
        match self {
            Self::Native(instance) => instance.run(ctx),
            Self::Program(instance) => {}
        }
    }
}

pub(crate) enum ParallelSystemInstance {
    Native(Box<dyn AnyNativeParallelSystemInstance>),
    Program(Program),
}

impl ParallelSystemInstance {
    pub(crate) fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), ResolverError> {
        match self {
            Self::Native(instance) => instance.resolve(resolver),
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run(&self, ctx: &Context) {
        match self {
            Self::Native(instance) => instance.run(ctx),
            Self::Program(instance) => {}
        }
    }
}

pub(crate) enum SystemInstance {
    Exclusive(ExclusiveSystemInstance),
    Parallel(ParallelSystemInstance),
}

pub(crate) struct SystemInstanceEntry {
    pub(crate) handle: ResourceRef,
    pub(crate) system: SystemInstance,
}

impl SystemInstanceEntry {
    pub(crate) fn setup(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        containers: &mut ContainerTable,
        resources: &mut ResourceManager,
    ) -> Result<(), ResolverError> {
        match self.system {
            SystemInstance::Exclusive(ref mut instance) => {
                instance.resolve(&mut ExclusiveResolver {
                    system: self.handle,
                    all: &mut Default::default(),
                    any: &mut Default::default(),
                    not: &mut Default::default(),
                    entities,
                    queries,
                    containers,
                    resources,
                })?;
            }
            SystemInstance::Parallel(ref mut instance) => {
                instance.resolve(&mut ParallelResolver {
                    system: self.handle,
                    reads: Vec::new(),
                    writes: Vec::new(),
                    all: &mut Default::default(),
                    any: &mut Default::default(),
                    not: &mut Default::default(),
                    entities,
                    queries,
                    containers,
                    resources,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct SystemTable {
    pub(crate) systems: Vec<(ResourceRef, SystemId)>,
    pub(crate) instances: SlotMap<SystemInstanceEntry>,
}

impl SystemTable {
    pub(crate) fn on_registry_update(
        &mut self,
        registry: &RegistryManager,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
    ) -> Result<(), RegistryError> {
        for id in registry.system.systems.keys() {
            // Create instance if missing
            if !self.entries.contains(id) {
                self.entries
                    .insert(id, SystemInstanceEntry::new(System(id), &registry.system));
            }

            // TODO: check if system must be changed
            if self.entries[id].dirty {
                self.entries[id].setup(&registry.component, entities, queries)?;
                self.entries[id].dirty = false;
            }
        }
        Ok(())
    }
}
