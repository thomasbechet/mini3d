use crate::{
    api::Context,
    feature::core::component::{Component, ComponentId},
};

use super::{
    archetype::Archetype,
    container::native::single::NativeSingleContainer,
    entity::{Entity, EntityEntry},
};

pub struct EntityBuilder<'a> {
    entity: Entity,
    archetype: Archetype,
    ctx: &'a mut Context<'a>,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(ctx: &'a mut Context<'a>) -> Self {
        // Find next entity
        let entity = ctx.entities.next_entity();
        Self {
            entity,
            archetype: ctx.entities.archetypes.empty,
            ctx,
        }
    }

    pub fn with<C: Component>(mut self, component: ComponentId, data: C) -> Self {
        self.archetype =
            self.ctx
                .entities
                .archetypes
                .find_add(self.ctx.queries, self.archetype, component);
        self.ctx.containers.entries[component.0]
            .container
            .get_mut()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<C>>()
            .unwrap()
            .add(self.entity, component);
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

    pub fn with_any(mut self, component: ComponentId) -> ComponentBuilder<'a> {}

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
        let archetype = &mut self.ctx.entities.archetypes[self.archetype];
        let pool_index = archetype.pool.len();
        archetype.pool.push(self.entity);
        // Update entity info
        self.ctx.entities.entries.set(
            self.entity.key(),
            EntityEntry {
                archetype: self.archetype,
                pool_index: pool_index as u32,
            },
        );
    }
}

pub struct ComponentBuilder<'a> {
    entity: Entity,
    archetype: Archetype,
    ctx: &'a mut Context<'a>,
}

impl<'a> ComponentBuilder<'a> {
    pub fn end(self) -> EntityBuilder<'a> {
        EntityBuilder {
            entity: self.entity,
            archetype: self.archetype,
            ctx: self.ctx,
        }
    }
}
