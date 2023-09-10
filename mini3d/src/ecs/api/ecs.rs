use crate::{
    ecs::{
        archetype::ArchetypeTable,
        container::ContainerTable,
        entity::{Entity, EntityBuilder, EntityTable},
        error::ECSError,
        query::{FilterQuery, Query, QueryTable},
        scheduler::{Invocation, Scheduler},
    },
    registry::component::ComponentHandle,
    utils::uid::UID,
};

pub struct ExclusiveECS<'a> {
    pub(crate) archetypes: &'a mut ArchetypeTable,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) cycle: u32,
}

impl<'a> ExclusiveECS<'a> {
    pub fn add(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(self.archetypes, self.entities, self.containers, self.cycle)
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities
            .remove(entity, self.archetypes, self.containers)
    }

    pub fn view<H: ComponentHandle>(&self, component: H) -> Result<H::ViewRef<'_>, ECSError> {
        self.containers.view(component)
    }

    pub fn view_mut<H: ComponentHandle>(&self, component: H) -> Result<H::ViewMut<'_>, ECSError> {
        self.containers.view_mut(component, self.cycle)
    }

    pub fn set_periodic_invoke(&mut self, stage: UID, frequency: f64) -> Result<(), ECSError> {
        self.scheduler.set_periodic_invoke(stage, frequency)
    }

    pub fn invoke(&mut self, stage: UID, invocation: Invocation) -> Result<(), ECSError> {
        self.scheduler.invoke(stage, invocation)
    }

    pub fn query(&self, query: Query) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub fn filter_query(&self, query: FilterQuery) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}

pub struct ParallelECS<'a> {
    pub(crate) containers: &'a ContainerTable,
    pub(crate) entities: &'a EntityTable,
    pub(crate) queries: &'a QueryTable,
    pub(crate) cycle: u32,
}

impl<'a> ParallelECS<'a> {
    pub fn view<H: ComponentHandle>(&self, component: H) -> Result<H::ViewRef<'_>, ECSError> {
        self.containers.view(component)
    }

    pub fn view_mut<H: ComponentHandle>(&self, component: H) -> Result<H::ViewMut<'_>, ECSError> {
        self.containers.view_mut(component, self.cycle)
    }

    pub fn query(&self, query: Query) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub fn filter_query(&self, query: FilterQuery) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}
