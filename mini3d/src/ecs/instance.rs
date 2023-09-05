use core::fmt::Display;

use crate::{
    registry::{
        component::{ComponentHandle, ComponentId, ComponentRegistry},
        error::RegistryError,
        system::{System, SystemRegistry},
    },
    utils::uid::UID,
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
    scheduler::SystemInstance,
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
        self.components.preallocate(handle, self.registry);
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
        self.components.preallocate(handle, self.registry);
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
        self.components.preallocate(handle, self.registry);
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

pub(crate) struct SystemInstanceEntry {
    pub(crate) system: System,
    pub(crate) instance: SystemInstance,
    pub(crate) last_execution_cycle: usize,
    pub(crate) filter_queries: Vec<FilterQuery>,
    pub(crate) active: bool,
}

impl SystemInstanceEntry {
    pub(crate) fn new(system: System, registry: &SystemRegistry) -> Self {
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
