use crate::{
    ecs::{
        entity::{Entity, EntityEntry},
        scheduler::Invocation,
    },
    utils::uid::ToUID,
};

use super::Context;

pub struct ECS;

impl ECS {
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

    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.entities.remove(entity, ctx.containers)
    }

    pub fn invoke(ctx: &mut Context, stage: impl ToUID, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }
}
