use crate::{
    feature::component::common::program::Program,
    registry::{
        component::{ComponentHandle, ComponentId, ComponentRegistry},
        error::RegistryError,
        system::{System, SystemRegistry},
        RegistryManager,
    },
    utils::{slotmap::SparseSecondaryMap, uid::ToUID},
};

use super::{
    api::{
        ecs::{ExclusiveECS, ParallelECS},
        ExclusiveAPI, ParallelAPI,
    },
    entity::EntityTable,
    query::{FilterQuery, QueryBuilder, QueryTable},
};

pub struct ExclusiveResolver<'a> {
    registry: &'a ComponentRegistry,
    system: System,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    filter_queries: &'a mut Vec<FilterQuery>,
}

impl<'a> ExclusiveResolver<'a> {
    pub fn find<H: ComponentHandle>(&mut self, component: impl ToUID) -> Result<H, RegistryError> {
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
            registry: self.registry,
            system: self.system,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            queries: self.queries,
            filter_queries: self.filter_queries,
        }
    }
}

pub struct ParallelResolver<'a> {
    registry: &'a ComponentRegistry,
    system: System,
    reads: Vec<ComponentId>,
    writes: Vec<ComponentId>,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    filter_queries: &'a mut Vec<FilterQuery>,
}

impl<'a> ParallelResolver<'a> {
    pub fn read<H: ComponentHandle>(&mut self, component: impl ToUID) -> Result<H, RegistryError> {
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

    pub fn write<H: ComponentHandle>(&mut self, component: impl ToUID) -> Result<H, RegistryError> {
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
            filter_queries: self.filter_queries,
        }
    }
}

pub(crate) trait AnyStaticExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI);
}

pub(crate) trait AnyStaticParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI);
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

    pub(crate) fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        match self {
            Self::Static(instance) => instance.run(ecs, api),
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

    pub(crate) fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) {
        match self {
            Self::Static(instance) => instance.run(ecs, api),
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
    pub(crate) filter_queries: Vec<FilterQuery>,
    pub(crate) active: bool,
    pub(crate) dirty: bool,
}

impl SystemInstanceEntry {
    fn new(system: System, registry: &SystemRegistry) -> Self {
        let instance = registry
            .get(system)
            .expect("System not found")
            .reflection
            .create_instance();
        Self {
            handle: system,
            system: instance,
            last_execution_cycle: 0,
            filter_queries: Vec::new(),
            active: true,
            dirty: true,
        }
    }

    pub(crate) fn setup(
        &mut self,
        registry: &ComponentRegistry,
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
                    filter_queries: &mut self.filter_queries,
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
                    filter_queries: &mut self.filter_queries,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct SystemInstanceTable {
    pub(crate) entries: SparseSecondaryMap<SystemInstanceEntry>,
}

impl SystemInstanceTable {
    pub(crate) fn on_registry_update(
        &mut self,
        registry: &RegistryManager,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
    ) -> Result<(), RegistryError> {
        for id in registry.systems.systems.keys() {
            // Create instance if missing
            if !self.entries.contains(id) {
                self.entries
                    .insert(id, SystemInstanceEntry::new(System(id), &registry.systems));
            }

            // TODO: check if system must be changed
            if self.entries[id].dirty {
                self.entries[id].setup(&registry.components, entities, queries)?;
                self.entries[id].dirty = false;
            }
        }
        Ok(())
    }
}
