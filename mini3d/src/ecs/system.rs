use core::fmt::Display;

use crate::{
    feature::component::common::program::Program,
    registry::{
        component::{ComponentId, ComponentRegistry},
        error::RegistryError,
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::{
    archetype::ArchetypeTable,
    context::{ExclusiveContext, ParallelContext},
    query::{GroupFilter, QueryBuilder, QueryTable},
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
    group_filters: Vec<GroupFilter>,
    queries: &'a mut QueryTable,
    archetypes: &'a mut ArchetypeTable,
}

impl<'a> ExclusiveResolver<'a> {
    pub fn find(&mut self, component: UID) -> Result<ComponentId, RegistryError> {
        self.registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })
    }

    pub fn query(&mut self) -> QueryBuilder<'a> {
        QueryBuilder {
            group_filters: &mut self.group_filters,
            queries: self.queries,
            archetypes: self.archetypes,
        }
    }
}

pub struct ParallelResolver<'a> {
    registry: &'a ComponentRegistry,
    group_filters: Vec<GroupFilter>,
    queries: &'a mut QueryTable,
    archetypes: &'a mut ArchetypeTable,
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

    pub fn query(&mut self) -> QueryBuilder<'a> {
        QueryBuilder {
            group_filters: &mut self.group_filters,
            queries: self.queries,
            archetypes: self.archetypes,
        }
    }
}

pub(crate) type SystemInstanceId = SlotId;

pub(crate) trait AnyStaticExclusiveSystemInstance {
    fn resolve(&mut self, resolver: &ExclusiveResolver) -> Result<(), RegistryError>;
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

struct SystemInstanceEntry {
    instance: SystemInstance,
    cycle: usize,
}

#[derive(Default)]
pub(crate) struct SystemTable {
    systems: SlotMap<SystemInstanceEntry>,
}
