use crate::{
    ecs::{
        builder::EntityBuilder,
        entity::Entity,
        query::QueryId,
        scheduler::Invocation,
        view::{ComponentViewMut, ComponentViewRef},
    },
    feature::core::component_type::{Component, ComponentId},
    utils::uid::{ToUID, UID},
};

use super::context::Context;

pub struct ECS;

impl ECS {
    pub fn create<'a>(ctx: &'a mut Context<'a>) -> EntityBuilder<'a> {
        EntityBuilder::new(ctx.entities, ctx.containers, ctx.queries, ctx.cycle)
    }

    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.entities.remove(entity, ctx.containers)
    }

    pub fn add<C: Component>(ctx: &mut Context, entity: Entity, component: ComponentId, data: C) {}

    pub fn remove(ctx: &mut Context, entity: Entity, component: ComponentId) {}

    pub fn view<V: ComponentViewRef>(ctx: &Context, id: ComponentId) -> V {
        ctx.containers.view(id)
    }

    pub fn view_mut<V: ComponentViewMut>(ctx: &Context, id: ComponentId) -> V {
        ctx.containers.view_mut(id, ctx.cycle)
    }

    pub fn set_periodic_invoke(ctx: &Context, stage: UID, frequency: f64) {
        ctx.scheduler.set_periodic_invoke(stage, frequency);
    }

    pub fn invoke(ctx: &mut Context, stage: impl ToUID, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }

    pub fn query<'a>(ctx: &'a Context, query: QueryId) -> impl Iterator<Item = Entity> + 'a {
        ctx.queries.entries[query.0]
            .archetypes
            .iter()
            .flat_map(|archetype| ctx.entities.iter_pool_entities(*archetype))
    }
}
