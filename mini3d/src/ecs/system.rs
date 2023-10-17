use crate::{feature::core::component_type::ComponentId, resource::ResourceManager};

use super::{api::context::Context, error::ResolverError};

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

use crate::{
    feature::common::program::Program,
    utils::{slotmap::SparseSecondaryMap, uid::ToUID},
};

use super::{
    api::ecs::ECS,
    entity::EntityTable,
    query::{QueryBuilder, QueryTable},
};

pub struct ExclusiveResolver<'a> {
    system: System,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    resources: &'a mut ResourceManager,
}

impl<'a> ExclusiveResolver<'a> {
    pub fn find(&mut self, component: impl ToUID) -> Result<ComponentId, ResolverError> {
        let handle = self
            .registry
            .find::<H>(component)
            .ok_or(RegistryError::ComponentNotFound)?;
        Ok(handle)
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
        }
    }
}

pub struct ParallelResolver<'a> {
    system: System,
    reads: Vec<ComponentId>,
    writes: Vec<ComponentId>,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    resources: &'a mut ResourceManager,
}

impl<'a> ParallelResolver<'a> {
    pub fn read(&mut self, component: impl ToUID) -> Result<ComponentId, RegistryError> {
        let handle: H = self
            .registry
            .find(component)
            .ok_or(RegistryError::ComponentNotFound)?;
        let id = handle.id();
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(H::new(id))
    }

    pub fn write(&mut self, component: impl ToUID) -> Result<ComponentId, RegistryError> {
        let handle: H = self
            .registry
            .find(component)
            .ok_or(RegistryError::ComponentNotFound)?;
        let id = handle.id();
        if self.reads.contains(&id) {
            self.reads.retain(|&x| x != id);
        }
        if !self.writes.contains(&id) {
            self.writes.push(id);
        }
        Ok(H::new(id))
    }

    pub fn query(&mut self) -> QueryBuilder<'_> {
        self.all.clear();
        self.any.clear();
        self.not.clear();
        QueryBuilder {
            registry: self.registry,
            system: self.system,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            queries: self.queries,
        }
    }
}

pub(crate) trait AnyStaticExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut Context);
}

pub(crate) trait AnyStaticParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &Context);
}

pub(crate) enum ExclusiveSystemInstance {
    Static(Box<dyn AnyStaticExclusiveSystemInstance>),
    Program(Program),
}

impl ExclusiveSystemInstance {
    pub(crate) fn resolve(
        &mut self,
        resolver: &mut ExclusiveResolver,
    ) -> Result<(), RegistryError> {
        match self {
            Self::Static(instance) => instance.resolve(resolver),
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        match self {
            Self::Static(instance) => instance.run(ctx),
            Self::Program(instance) => {}
        }
    }
}

pub(crate) enum ParallelSystemInstance {
    Static(Box<dyn AnyStaticParallelSystemInstance>),
    Program(Program),
}

impl ParallelSystemInstance {
    pub(crate) fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        match self {
            Self::Static(instance) => instance.resolve(resolver),
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run(&self, ctx: &Context) {
        match self {
            Self::Static(instance) => instance.run(ctx),
            Self::Program(instance) => {}
        }
    }
}

pub(crate) enum SystemInstance {
    Exclusive(ExclusiveSystemInstance),
    Parallel(ParallelSystemInstance),
}

pub(crate) struct SystemInstanceEntry {
    pub(crate) handle: System,
    pub(crate) system: SystemInstance,
    pub(crate) last_execution_cycle: usize,
    pub(crate) active: bool,
    pub(crate) dirty: bool,
}

impl SystemInstanceEntry {
    fn new(system: System, registry: &SystemRegistryManager) -> Self {
        let instance = registry
            .get(system)
            .expect("System not found")
            .reflection
            .create_instance();
        Self {
            handle: system,
            system: instance,
            last_execution_cycle: 0,
            active: true,
            dirty: true,
        }
    }

    pub(crate) fn setup(
        &mut self,
        registry: &ComponentRegistryManager,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
    ) -> Result<(), RegistryError> {
        match self.system {
            SystemInstance::Exclusive(ref mut instance) => {
                instance.resolve(&mut ExclusiveResolver {
                    registry,
                    system: self.handle,
                    all: &mut Default::default(),
                    any: &mut Default::default(),
                    not: &mut Default::default(),
                    entities,
                    queries,
                })?;
            }
            SystemInstance::Parallel(ref mut instance) => {
                instance.resolve(&mut ParallelResolver {
                    registry,
                    system: self.handle,
                    reads: Vec::new(),
                    writes: Vec::new(),
                    all: &mut Default::default(),
                    any: &mut Default::default(),
                    not: &mut Default::default(),
                    entities,
                    queries,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct SystemInstanceManager {
    pub(crate) entries: SparseSecondaryMap<SystemInstanceEntry>,
}

impl SystemInstanceManager {
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
