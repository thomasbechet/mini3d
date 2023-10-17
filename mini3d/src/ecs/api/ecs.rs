use std::task::Context;

use crate::{
    ecs::{
        builder::EntityBuilder,
        entity::Entity,
        query::Query,
        scheduler::Invocation,
        view::{ComponentViewMut, ComponentViewRef},
    },
    resource::handle::ResourceHandle,
    utils::uid::{ToUID, UID},
};

pub struct ECS;

impl ECS {
    pub fn create<'a>(ctx: &'a mut Context<'a>) -> EntityBuilder<'a> {
        EntityBuilder::new(ctx.entities, ctx.containers, ctx.queries, ctx.cycle)
    }

    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.entities.remove(entity, ctx.containers)
    }

    pub fn add<C: Component>(ctx: &mut Context, entity: Entity, component: ComponentType, data: C) {
    }

    pub fn remove(ctx: &mut Context, entity: Entity, component: ComponentType) {}

    pub fn view<V: ComponentViewRef>(ctx: &Context, ty: ComponentType) -> V {
        ctx.containers.view(ty)
    }

    pub fn view_mut<V: ComponentViewMut>(ctx: &Context, ty: ComponentType) -> V {
        ctx.containers.view_mut(ty, ctx.cycle)
    }

    pub fn set_periodic_invoke(ctx: &Context, stage: UID, frequency: f64) {
        ctx.scheduler.set_periodic_invoke(stage, frequency);
    }

    pub fn invoke(
        ctx: &mut Context,
        stage: impl ToUID,
        invocation: Invocation,
    ) -> Result<(), RegistryError> {
        ctx.scheduler.invoke(stage, invocation)
    }

    pub fn query<'a>(ctx: &'a Context, query: Query) -> impl Iterator<Item = Entity> + 'a {
        ctx.queries.entries[query.0]
            .archetypes
            .iter()
            .flat_map(|archetype| ctx.entities.iter_pool_entities(*archetype))
    }

    pub fn add_system(ctx: &mut Context, system: ResourceHandle) -> Result<(), RegistryError> {
        ctx.scheduler.add_system(system)
    }

    pub fn remove_system(ctx: &mut Context, system: ResourceHandle) -> Result<(), RegistryError> {
        ctx.scheduler.remove_system(system)
    }
}
