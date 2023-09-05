use core::fmt::Display;

use crate::{
    registry::{
        component::{ComponentHandle, ComponentId, ComponentRegistry},
        error::RegistryError,
        system::System,
    },
    utils::{slotmap::SecondaryMap, uid::UID},
};

use super::{
    api::{
        ecs::{ExclusiveECS, ParallelECS},
        ExclusiveAPI, ParallelAPI,
    },
    archetype::ArchetypeTable,
    component::ComponentTable,
    entity::EntityTable,
    error::SceneError,
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
    pub(crate) last_execution_cycle: usize,
    pub(crate) filter_queries: Vec<FilterQuery>,
    pub(crate) active: bool,
}

pub(crate) struct SystemInstanceTable {
    pub(crate) instances: SecondaryMap<SystemInstanceEntry>,
}

impl SystemInstanceTable {
    pub(crate) fn set_active(&mut self, system: System, active: bool) -> Result<(), RegistryError> {
        self.instances
            .get_mut(system.into())
            .ok_or(ECS::SystemNotFound)?
            .active = active;
        Ok(())
    }
}

impl Default for SystemInstanceTable {
    fn default() -> Self {
        // Create table
        let mut table = Self {
            instances: Default::default(),
        };
        // Prepare default stages
        table
            .add_stage(SystemStage::UPDATE, SystemStage::update())
            .unwrap();
        table
            .add_stage(
                SystemStage::FIXED_UPDATE_60HZ,
                SystemStage::fixed_update(1.0 / 60.0),
            )
            .unwrap();
        table
    }
}
