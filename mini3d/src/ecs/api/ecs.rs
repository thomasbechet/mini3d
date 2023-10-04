use crate::{
    ecs::{
        container::ContainerTable,
        entity::{Entity, EntityBuilder, EntityTable},
        error::ECSError,
        query::{FilterQuery, Query, QueryTable},
        scheduler::{Invocation, Scheduler},
    },
    registry::{component::ComponentTypeTrait, error::RegistryError},
    resource::handle::ResourceHandle,
    utils::uid::{ToUID, UID},
};

pub struct ECS<'a> {
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) cycle: u32,
}

impl<'a> ECS<'a> {
    pub fn add(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(self.entities, self.containers, self.queries, self.cycle)
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities.remove(entity, self.containers)
    }

    pub fn view<H: ComponentTypeTrait>(&self, component: H) -> H::SingleViewRef<'_> {
        self.containers
            .view(component)
            .unwrap_or_else(|_| panic!("{}", ECSError::ContainerBorrowMut.to_string()))
    }

    pub fn view_mut<H: ComponentTypeTrait>(&self, component: H) -> H::SingleViewMut<'_> {
        self.containers
            .view_mut(component, self.cycle)
            .unwrap_or_else(|_| panic!("{}", ECSError::ContainerBorrowMut.to_string()))
    }

    pub fn set_periodic_invoke(&mut self, stage: UID, frequency: f64) {
        self.scheduler.set_periodic_invoke(stage, frequency);
    }

    pub fn invoke(
        &mut self,
        stage: impl ToUID,
        invocation: Invocation,
    ) -> Result<(), RegistryError> {
        self.scheduler.invoke(stage, invocation)
    }

    pub fn query(&self, query: Query) -> impl Iterator<Item = Entity> + '_ {
        self.queries.entries[query.0]
            .archetypes
            .iter()
            .flat_map(|archetype| self.entities.iter_pool_entities(*archetype))
    }

    pub fn query_filter(&self, query: FilterQuery) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_queries[query.0].pool.iter().copied()
    }

    pub fn add_system(&mut self, system: ResourceHandle) -> Result<(), RegistryError> {
        self.scheduler.add_system(system)
    }

    pub fn remove_system(&mut self, system: ResourceHandle) -> Result<(), RegistryError> {
        self.scheduler.remove_system(system)
    }
}
