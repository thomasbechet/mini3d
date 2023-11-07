use crate::{
    ecs::{
        entity::{Entity, EntityChange},
        scheduler::Invocation,
        system::{ExclusiveSystem, ParallelSystem},
    },
    feature::ecs::{
        component::{ComponentType, ComponentTypeHandle},
        system::{
            System, SystemHandle, SystemSet, SystemSetHandle, SystemStage, SystemStageHandle,
        },
    },
    resource::error::ResourceError,
    utils::uid::ToUID,
};

use super::Context;

impl Entity {
    pub fn create(ctx: &mut Context) -> Entity {
        let entity = ctx.entities.generate_entity();
        ctx.entities.changes.push(EntityChange::Created(entity));
        entity
    }

    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.entities.changes.push(EntityChange::Destroyed(entity));
    }
}

impl System {
    pub fn create_native_exclusive<S: ExclusiveSystem>(
        ctx: &mut Context,
        key: &str,
    ) -> Result<SystemHandle, ResourceError> {
        ctx.resource
            .create(
                Some(key),
                ctx.ecs_types.system,
                ctx.activity.active,
                System::native_exclusive::<S>(),
            )
            .map(SystemHandle)
    }

    pub fn create_native_parallel<S: ParallelSystem>(
        ctx: &mut Context,
        key: &str,
    ) -> Result<SystemHandle, ResourceError> {
        ctx.resource
            .create(
                Some(key),
                ctx.ecs_types.system,
                ctx.activity.active,
                System::native_parallel::<S>(),
            )
            .map(SystemHandle)
    }
}

impl SystemSet {
    pub fn create(
        ctx: &mut Context,
        key: &str,
        set: SystemSet,
    ) -> Result<SystemSetHandle, ResourceError> {
        ctx.resource
            .create(
                Some(key),
                ctx.ecs_types.system_set,
                ctx.activity.active,
                set,
            )
            .map(SystemSetHandle)
    }
}

impl SystemStage {
    pub fn invoke(ctx: &mut Context, stage: SystemStageHandle, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<SystemStageHandle> {
        ctx.resource.find_typed(key, ctx.ecs_types.system_stage)
    }
}

impl ComponentType {
    pub fn create(
        ctx: &mut Context,
        key: &str,
        ty: ComponentType,
    ) -> Result<ComponentTypeHandle, ResourceError> {
        ctx.resource
            .create(Some(key), ctx.ecs_types.component, ctx.activity.active, ty)
            .map(ComponentTypeHandle)
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ComponentTypeHandle> {
        ctx.resource.find_typed(key, ctx.ecs_types.component)
    }
}
