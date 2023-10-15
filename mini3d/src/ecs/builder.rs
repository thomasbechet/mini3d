use crate::feature::common::component_type::{Component, ComponentId, PrivateComponentTableMut};

use super::{
    archetype::Archetype,
    container::ContainerTable,
    entity::{Entity, EntityEntry, EntityTable},
    query::QueryTable,
};

pub struct EntityBuilder<'a> {
    entity: Entity,
    archetype: Archetype,
    entities: &'a mut EntityTable,
    containers: &'a mut ContainerTable,
    queries: &'a mut QueryTable,
    cycle: u32,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(
        entities: &'a mut EntityTable,
        containers: &'a mut ContainerTable,
        queries: &'a mut QueryTable,
        cycle: u32,
    ) -> Self {
        // Find next entity
        let entity = entities.next_entity();
        Self {
            entity,
            archetype: entities.archetypes.empty,
            entities,
            containers,
            queries,
            cycle,
        }
    }

    pub fn with<C: Component>(mut self, component: ComponentId, data: C) -> Self {
        self.archetype = self
            .entities
            .archetypes
            .find_add(self.queries, self.archetype, component);
        component.insert_single_container(
            PrivateComponentTableMut(self.containers),
            self.entity,
            data,
            self.cycle,
        );
        self
    }

    pub fn with_array<C: Component>(mut self, component: ComponentId, data: &[C]) -> Self {
        self
    }

    pub fn with_list<C: Component>(mut self, component: ComponentId, data: &[C]) -> Self {
        self
    }

    pub fn with_map<C: Component>(mut self, component: ComponentId, data: &[(Entity, C)]) -> Self {
        self
    }

    pub fn with_any(mut self, component: ComponentId) -> AnyComponentBuilder<'a> {}

    pub fn with_default(self, component: ComponentId) -> Self {
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}

impl<'a> Drop for EntityBuilder<'a> {
    fn drop(&mut self) {
        // Add to pool
        let archetype = &mut self.entities.archetypes[self.archetype];
        let pool_index = archetype.pool.len();
        archetype.pool.push(self.entity);
        // Add to added filter queries
        for added in &archetype.added_filter_queries {
            self.queries.filter_queries[added.0].pool.push(self.entity);
        }
        // Update entity info
        self.entities.entries.set(
            self.entity.key(),
            EntityEntry {
                archetype: self.archetype,
                pool_index: pool_index as u32,
            },
        );
    }
}

pub struct AnyComponentBuilder<'a> {
    entity: Entity,
    archetype: Archetype,
    entities: &'a mut EntityTable,
    containers: &'a mut ContainerTable,
    queries: &'a mut QueryTable,
    cycle: u32,
}

impl<'a> AnyComponentBuilder<'a> {
    pub fn end(self) -> EntityBuilder<'a> {
        EntityBuilder {
            entity: self.entity,
            archetype: self.archetype,
            entities: self.entities,
            containers: self.containers,
            queries: self.queries,
            cycle: self.cycle,
        }
    }
}
