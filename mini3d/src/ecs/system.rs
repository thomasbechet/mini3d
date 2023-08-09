use core::fmt::Display;

use crate::{
    feature::component::common::program::Program,
    registry::{
        component::{ComponentId, ComponentRegistry},
        error::RegistryError,
        system::{SystemId, SystemRegistry},
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::{
    archetype::ArchetypeTable,
    component::{ComponentHandle, ComponentTable},
    context::{ExclusiveContext, ParallelContext},
    entity::EntityTable,
    error::SceneError,
    query::{FilterQueryId, QueryBuilder, QueryTable},
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
        let id = self
            .registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        self.components.preallocate(id, self.registry);
        Ok(H::new(component, id))
    }

    pub fn query(&mut self) -> QueryBuilder<'a> {
        QueryBuilder {
            registry: self.registry,
            system: self.system,
            all: &mut self.all,
            any: &mut self.any,
            not: &mut self.not,
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
        let id = self
            .registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        self.components.preallocate(id, self.registry);
        if !self.reads.contains(&id) && !self.writes.contains(&id) {
            self.reads.push(id);
        }
        Ok(H::new(component, id))
    }

    pub fn write<H: ComponentHandle>(&mut self, component: UID) -> Result<H, RegistryError> {
        let id = self
            .registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        self.components.preallocate(id, self.registry);
        if self.reads.contains(&id) {
            self.reads.retain(|&x| x != id);
        }
        if !self.writes.contains(&id) {
            self.writes.push(id);
        }
        Ok(H::new(component, id))
    }

    pub fn query(&mut self) -> QueryBuilder<'a> {
        QueryBuilder {
            registry: self.registry,
            system: self.system,
            all: &mut self.all,
            any: &mut self.any,
            not: &mut self.not,
            entities: self.entities,
            archetypes: self.archetypes,
            queries: self.queries,
        }
    }
}

pub(crate) trait AnyStaticExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult;
}

pub(crate) trait AnyStaticParallelSystemInstance {
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ParallelContext) -> SystemResult;
}

pub(crate) enum StaticSystemInstance {
    Exclusive(Box<dyn AnyStaticExclusiveSystemInstance>),
    Parallel(Box<dyn AnyStaticParallelSystemInstance>),
}

struct ProgramSystemInstance {
    program: Program,
}

pub(crate) enum SystemInstance {
    Static(StaticSystemInstance),
    Program(ProgramSystemInstance),
}

type SystemInstanceId = SlotId;
pub(crate) type SystemStageId = SlotId;
type SystemGroupId = SlotId;

pub(crate) struct SystemInstanceEntry {
    pub(crate) name: String,
    pub(crate) instance: SystemInstance,
    pub(crate) last_execution_cycle: usize,
    pub(crate) filter_queries: Vec<FilterQueryId>,
    pub(crate) active: bool,
    pub(crate) next_instance: Option<SystemInstanceId>,
    pub(crate) prev_instance: Option<SystemInstanceId>,
}

pub enum StageEvent {
    SceneChanged,
}

pub(crate) enum SystemStageKind {
    Update,
    FixedUpdate(f64),
    Event(StageEvent),
}

pub struct SystemStage {
    pub(crate) name: String,
    pub(crate) kind: SystemStageKind,
}

impl SystemStage {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE_60HZ: &'static str = "fixed_update_60hz";
    pub const SCENE_CHANGED: &'static str = "scene_changed";

    fn update() -> Self {
        Self {
            name: Self::UPDATE.to_string(),
            kind: SystemStageKind::Update,
        }
    }

    pub(crate) fn frequency(&self) -> Option<f64> {
        match self.kind {
            SystemStageKind::FixedUpdate(frequency) => Some(frequency),
            _ => None,
        }
    }

    pub fn fixed_update(name: &str, frequency: f64) -> Self {
        Self {
            name: name.to_string(),
            kind: SystemStageKind::FixedUpdate(frequency),
        }
    }

    pub fn event(name: &str, event: StageEvent) -> Self {
        Self {
            name: name.to_string(),
            kind: SystemStageKind::Event(event),
        }
    }
}

pub(crate) struct StageEntry {
    pub(crate) stage: SystemStage,
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
            .find(|(_, entry)| UID::new(&entry.stage.name) == stage)
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

    pub(crate) fn add_stage(&mut self, stage: SystemStage) -> Result<(), SceneError> {
        if self.find_stage(UID::new(&stage.name)).is_some() {
            return Err(SceneError::SystemStageAlreadyExists);
        }
        self.stages.add(StageEntry {
            stage,
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
        let reg_id = registry.find(system).ok_or(SceneError::SystemNotFound)?;
        let instance = registry.get(reg_id).unwrap().reflection.create_instance();
        // Create new entry
        let id = self.instances.add(SystemInstanceEntry {
            name: name.to_string(),
            instance,
            active: true,
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
        let mut table = Self {
            instances: Default::default(),
            stages: Default::default(),
        };
        table.add_stage(SystemStage::update()).unwrap();
        table
            .add_stage(SystemStage::fixed_update(
                SystemStage::FIXED_UPDATE_60HZ,
                1.0 / 60.0,
            ))
            .unwrap();
        table
            .add_stage(SystemStage::event(
                SystemStage::SCENE_CHANGED,
                StageEvent::SceneChanged,
            ))
            .unwrap();
        table
    }
}
