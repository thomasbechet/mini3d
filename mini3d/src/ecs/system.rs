use core::fmt::Display;

use crate::{
    registry::{
        component::{ComponentHandle, ComponentId, ComponentRegistry},
        error::RegistryError,
        system::SystemRegistry,
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::UID,
    },
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
    system: SystemId,
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
    system: SystemId,
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

pub(crate) type SystemInstanceId = SlotId;
pub(crate) type SystemStageId = SlotId;

pub(crate) struct SystemInstanceEntry {
    pub(crate) system: SystemId,
    pub(crate) last_execution_cycle: usize,
    pub(crate) filter_queries: Vec<FilterQuery>,
    pub(crate) active: bool,
    pub(crate) next_instance: Option<SystemInstanceId>,
    pub(crate) prev_instance: Option<SystemInstanceId>,
}

pub enum StageEvent {}

pub(crate) struct StageEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_STAGE_NAME_LEN>,
    pub(crate) kind: SystemStageKind,
    pub(crate) first_instance: Option<SystemInstanceId>,
}

pub(crate) struct SystemTable {
    pub(crate) stages: SlotMap<StageEntry>,
    pub(crate) instances: SlotMap<SystemInstanceEntry>,
}

pub struct SystemOrder {}

impl SystemOrder {
    pub fn new() -> Self {
        Self {}
    }
}

impl SystemTable {
    pub(crate) fn find_stage(&self, stage: UID) -> Option<SystemStageId> {
        self.stages
            .iter()
            .find(|(_, entry)| UID::new(&entry.name) == stage)
            .map(|(id, _)| id)
    }

    fn find_system_in_stage(&self, stage: SystemStageId, system: UID) -> Option<SystemInstanceId> {
        let mut instance = self.stages[stage].first_instance;
        while let Some(id) = instance {
            let entry = &self.instances[id];
            if UID::new(&entry.name) == system {
                return Some(id);
            }
            instance = entry.next_instance;
        }
        None
    }

    fn find_last_system_in_stage(&self, stage: SystemStageId) -> Option<SystemInstanceId> {
        let mut instance = self.stages[stage].first_instance;
        while let Some(id) = instance {
            let entry = &self.instances[id];
            if entry.next_instance.is_none() {
                return Some(id);
            }
            instance = entry.next_instance;
        }
        None
    }

    pub(crate) fn add_stage(&mut self, name: &str, stage: SystemStage) -> Result<(), SceneError> {
        if self.find_stage(UID::new(name)).is_some() {
            return Err(SceneError::SystemStageAlreadyExists);
        }
        self.stages.add(StageEntry {
            name: name.into(),
            kind: stage.kind,
            first_instance: None,
        });
        Ok(())
    }

    pub(crate) fn remove_stage(&mut self, stage: UID) -> Result<(), SceneError> {
        let stage = self
            .find_stage(stage)
            .ok_or(SceneError::SystemStageNotFound)?;
        let mut instance = self.stages[stage].first_instance;
        while let Some(id) = instance {
            let entry = &self.instances[id];
            instance = entry.next_instance;
            self.instances.remove(id);
        }
        self.stages.remove(stage);
        Ok(())
    }

    pub(crate) fn add_system(
        &mut self,
        registry: &SystemRegistry,
        name: &str,
        system: UID,
        stage: UID,
        order: SystemOrder,
    ) -> Result<(), SceneError> {
        // Find stage
        let stage = self
            .find_stage(stage)
            .ok_or(SceneError::SystemStageNotFound)?;
        // Check existing system
        if self.find_system_in_stage(stage, system).is_some() {
            return Err(SceneError::SystemAlreadyExists);
        }
        // Find system
        let system = registry.find(system).ok_or(SceneError::SystemNotFound)?;
        // Create new entry
        let id = self.instances.add(SystemInstanceEntry {
            name: name.into(),
            active: true,
            system,
            last_execution_cycle: 0,
            filter_queries: Vec::new(),
            next_instance: None,
            prev_instance: None,
        });
        // Find position
        if let Some(last) = self.find_last_system_in_stage(stage) {
            self.instances[last].next_instance = Some(id);
            self.instances[id].prev_instance = Some(last);
        } else {
            self.stages[stage].first_instance = Some(id);
        }
        Ok(())
    }

    pub(crate) fn remove_system(&mut self, stage: UID, system: UID) -> Result<(), SceneError> {
        let stage = self
            .find_stage(stage)
            .ok_or(SceneError::SystemStageNotFound)?;
        let system = self
            .find_system_in_stage(stage, system)
            .ok_or(SceneError::SystemNotFound)?;
        if let Some(prev) = self.instances[system].prev_instance {
            self.instances[prev].next_instance = self.instances[system].next_instance;
        } else {
            self.stages[stage].first_instance = self.instances[system].next_instance;
        }
        self.instances.remove(system);
        Ok(())
    }

    pub(crate) fn set_active(
        &mut self,
        stage: UID,
        system: UID,
        active: bool,
    ) -> Result<(), SceneError> {
        let stage = self
            .find_stage(stage)
            .ok_or(SceneError::SystemStageNotFound)?;
        let system = self
            .find_system_in_stage(stage, system)
            .ok_or(SceneError::SystemNotFound)?;
        self.instances[system].active = active;
        Ok(())
    }
}

impl Default for SystemTable {
    fn default() -> Self {
        // Create table
        let mut table = Self {
            instances: Default::default(),
            stages: Default::default(),
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
