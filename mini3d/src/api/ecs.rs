use crate::{
    ecs::{
        entity::{Entity, EntityEntry},
    },
};

use super::Context;

pub struct ECS;

impl ECS {
    /// The created entity should not be visible in iteration
    /// This operation doesn't block iteration
    pub fn create(ctx: &mut Context) -> Entity {
        // Add to pool
        let entity = ctx.entities.next_entity();
        let archetype = &mut ctx.entities.archetypes[ctx.entities.archetypes.empty];
        let pool_index = archetype.pool.len();
        archetype.pool.push(entity);
        // Update entity info
        ctx.entities.entries.set(
            entity.key(),
            EntityEntry {
                archetype,
                pool_index: pool_index as u32,
            },
        );
        entity
    }

    /// The destroyed entity should not be visible in iteration
    /// This operation doesn't block iteration
    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.entities.add_to_remove_queue(entity);
    }

    pub fn add(ctx: &mut Context, entity: Entity)

    pub fn remove(ctx: &mut Context, )

    pub fn invoke(ctx: &mut Context, stage: impl ToUID, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }
}
