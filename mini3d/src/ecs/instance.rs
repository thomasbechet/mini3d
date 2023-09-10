use core::fmt::Display;

use std::ops::Index;

use crate::{
    feature::component::common::program::Program,
    registry::{
        component::{ComponentHandle, ComponentId, ComponentRegistry},
        error::RegistryError,
        system::{System, SystemRegistry},
        RegistryManager,
    },
    utils::{slotmap::SparseSecondaryMap, uid::UID},
};

use super::{
    api::{
        ecs::{ExclusiveECS, ParallelECS},
        ExclusiveAPI, ParallelAPI,
    },
    archetype::ArchetypeTable,
    component::ComponentTable,
    entity::EntityTable,
    query::{FilterQuery, QueryBuilder, QueryTable},
};

pub trait SystemError: Display {}

pub type SystemResult = Result<(), Box<dyn SystemError>>;

impl SystemError for &str {}
impl SystemError for String {}
impl From<&str> for Box<dyn SystemError> {
    fn from(error: &str) -> Self {
        Box::new(error.to_string())
    }
}

pub struct ExclusiveResolver<'a> {
    registry: &'a ComponentRegistry,
    system: System,
    all: &'a mut Vec<ComponentId>,
    any: &'a mut Vec<ComponentId>,
    not: &'a mut Vec<ComponentId>,
    components: &'a mut ComponentTable,
    entities: &'a mut EntityTable,
    archetypes: &'a mut ArchetypeTable,
    queries: &'a mut QueryTable,
}

impl<'a> ExclusiveResolver<'a> {
    pub fn find<H: ComponentHandle>(&mut self, component: UID) -> Result<H, RegistryError> {
        let handle = self
            .registry
            .find::<H>(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        Ok(handle)
    }

    pub fn query(&mut self) -> QueryBuilder<'_> {
        QueryBuilder {
            registry: self.registry,
            system: self.system,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            archetypes: self.archetypes,
            queries: self.queries,
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
    components: &'a mut ComponentTable,
    entities: &'a mut EntityTable,
    archetypes: &'a mut ArchetypeTable,
    queries: &'a mut QueryTable,
}

impl<'a> ParallelResolver<'a> {
    pub fn read<H: ComponentHandle>(&mut self, component: UID) -> Result<H, RegistryError> {
        let handle: H = self
            .registry
            .find(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        let id = handle.id();
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(H::new(id))
    }

    pub fn write<H: ComponentHandle>(&mut self, component: UID) -> Result<H, RegistryError> {
        let handle: H = self
            .registry
            .find(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
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
        QueryBuilder {
            registry: self.registry,
            system: self.system,
            all: self.all,
            any: self.any,
            not: self.not,
            entities: self.entities,
            archetypes: self.archetypes,
            queries: self.queries,
        }
    }
}

pub(crate) trait AnyStaticExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult;
}

pub(crate) trait AnyStaticParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult;
}

pub(crate) enum StaticSystemInstance {
    Exclusive(Box<dyn AnyStaticExclusiveSystemInstance>),
    Parallel(Box<dyn AnyStaticParallelSystemInstance>),
}

pub(crate) struct ProgramSystemInstance {
    program: Program,
}

pub(crate) enum SystemInstance {
    Static(StaticSystemInstance),
    Program(ProgramSystemInstance),
}

impl SystemInstance {
    pub(crate) fn resolve_exclusive(
        &mut self,
        resolver: &mut ExclusiveResolver,
    ) -> Result<(), RegistryError> {
        match self {
            Self::Static(instance) => match instance {
                StaticSystemInstance::Exclusive(instance) => instance.resolve(resolver),
                StaticSystemInstance::Parallel(_) => unreachable!(),
            },
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run_exclusive(
        &self,
        ecs: &mut ExclusiveECS,
        api: &mut ExclusiveAPI,
    ) -> SystemResult {
        match self {
            Self::Static(instance) => match instance {
                StaticSystemInstance::Exclusive(instance) => instance.run(ecs, api),
                StaticSystemInstance::Parallel(_) => unreachable!(),
            },
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn resolve_parallel(
        &mut self,
        resolver: &mut ParallelResolver,
    ) -> Result<(), RegistryError> {
        match self {
            Self::Static(instance) => match instance {
                StaticSystemInstance::Parallel(instance) => instance.resolve(resolver),
                StaticSystemInstance::Exclusive(_) => unreachable!(),
            },
            Self::Program(_) => Ok(()),
        }
    }

    pub(crate) fn run_parallel(
        &self,
        ecs: &mut ParallelECS,
        api: &mut ParallelAPI,
    ) -> SystemResult {
        match self {
            Self::Static(instance) => match instance {
                StaticSystemInstance::Parallel(instance) => instance.run(ecs, api),
                StaticSystemInstance::Exclusive(_) => unreachable!(),
            },
            Self::Program(instance) => Ok(()),
        }
    }
}

pub(crate) struct SystemInstanceEntry {
    pub(crate) system: System,
    pub(crate) instance: SystemInstance,
    pub(crate) last_execution_cycle: usize,
    pub(crate) filter_queries: Vec<FilterQuery>,
    pub(crate) active: bool,
}

impl SystemInstanceEntry {
    fn new(system: System, registry: &SystemRegistry) -> Self {
        let instance = registry
            .get(system)
            .expect("System not found")
            .reflection
            .create_instance();
        Self {
            system,
            instance,
            last_execution_cycle: 0,
            filter_queries: Vec::new(),
            active: true,
        }
    }
}

#[derive(Default)]
pub(crate) struct SystemInstanceTable {
    instances: SparseSecondaryMap<SystemInstanceEntry>,
}

impl SystemInstanceTable {
    pub(crate) fn on_registry_update(
        &mut self,
        registry: &RegistryManager,
        components: &mut ComponentTable,
        entities: &mut EntityTable,
        archetypes: &mut ArchetypeTable,
        queries: &mut QueryTable,
    ) -> Result<(), RegistryError> {
        for (id, entry) in registry.systems.systems.iter() {
            // Create instance if missing
            if !self.instances.contains(id) {
                let instance = registry
                    .systems
                    .get(id.into())
                    .expect("System not found")
                    .reflection
                    .create_instance();
                let instance = SystemInstanceEntry {
                    system: id.into(),
                    instance,
                    last_execution_cycle: 0,
                    filter_queries: Vec::new(),
                    active: entry.active_by_default,
                };
                self.instances.insert(id, instance);
            }

            // Resolve instance
            self.instances[id]
                .instance
                .resolve_exclusive(&mut ExclusiveResolver {
                    registry: &registry.components,
                    system: id.into(),
                    all: &mut Default::default(),
                    any: &mut Default::default(),
                    not: &mut Default::default(),
                    components,
                    entities,
                    archetypes,
                    queries,
                })?;
        }
        Ok(())
    }
}

impl Index<System> for SystemInstanceTable {
    type Output = SystemInstance;
    fn index(&self, id: System) -> &Self::Output {
        &self.instances[id.into()].instance
    }
}
