use crate::{
    ecs::{
        entity::{Entity, EntityChange},
        scheduler::Invocation,
    },
    feature::core::system::SystemStage,
    utils::uid::ToUID,
};

use super::Context;

impl Entity {
    pub fn add(ctx: &mut Context) -> Entity {
        let entity = ctx.entities.generate_entity();
        ctx.entities.changes.push(EntityChange::Added(entity));
        entity
    }

    pub fn remove(ctx: &mut Context, entity: Entity) {
        ctx.entities.changes.push(EntityChange::Removed(entity));
    }
}

impl SystemStage {
    pub fn invoke(ctx: &mut Context, stage: impl ToUID, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }
}
