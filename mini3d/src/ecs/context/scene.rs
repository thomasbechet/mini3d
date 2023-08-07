use crate::{
    ecs::{
        archetype::ArchetypeTable,
        component::ComponentTable,
        entity::{Entity, EntityBuilder, EntityTable},
        error::SceneError,
        query::{FilterQueryId, QueryId, QueryTable},
        view::{ComponentViewMut, ComponentViewRef},
    },
    registry::{component::ComponentId, RegistryManager},
    utils::uid::UID,
};

pub struct ExclusiveSceneContext<'a> {
    registry: &'a RegistryManager,
    archetypes: &'a mut ArchetypeTable,
    components: &'a mut ComponentTable,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    cycle: u32,
}

impl<'a> ExclusiveSceneContext<'a> {
    pub fn add(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(
            &self.registry.components,
            &mut self.archetypes,
            &mut self.entities,
            &mut self.components,
            self.cycle,
        )
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities
            .remove(entity, self.archetypes, self.components)
    }

    pub fn view(&self, component: ComponentId) -> Result<ComponentViewRef<'_>, SceneError> {
        self.components.view(component)
    }

    pub fn view_mut(&self, component: ComponentId) -> Result<ComponentViewMut<'_>, SceneError> {
        self.components.view_mut(component, self.cycle)
    }

    pub(crate) fn query(&self, query: QueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub(crate) fn filter_query(&self, query: FilterQueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}

pub struct ParallelSceneContext<'a> {
    uid: UID,
    registry: &'a RegistryManager,
    components: &'a ComponentTable,
    entities: &'a EntityTable,
    queries: &'a QueryTable,
    cycle: u32,
}

impl<'a> ParallelSceneContext<'a> {
    pub fn view(&self, component: ComponentId) -> Result<ComponentViewRef<'_>, SceneError> {
        self.components.view(component)
    }

    pub fn view_mut(&self, component: ComponentId) -> Result<ComponentViewMut<'_>, SceneError> {
        self.components.view_mut(component, self.cycle)
    }

    pub(crate) fn query(&self, query: QueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub(crate) fn filter_query(&self, query: FilterQueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}
