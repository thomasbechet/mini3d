use crate::{
    ecs::resource::{
        component::{ComponentType, ComponentTypeHandle},
        system::{
            System, SystemHandle, SystemSet, SystemSetHandle, SystemStage, SystemStageHandle,
        },
    },
    ecs::{
        entity::Entity,
        scheduler::Invocation,
        system::{ExclusiveSystem, ParallelSystem},
        ECSCommand,
    },
    resource::error::ResourceError,
    utils::uid::ToUID,
};

use super::Context;

impl Entity {
    pub fn create(ctx: &mut Context) -> Entity {
        let entity = ctx.entities.generate_entity();
        ctx.entity_created.push(entity);
        entity
    }

    pub fn destroy(ctx: &mut Context, entity: Entity) {
        ctx.entity_destroyed.push(entity);
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
                System::native_parallel::<S>(),
            )
            .map(SystemHandle)
    }

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<SystemHandle> {
        ctx.resource.find_typed(name, ctx.ecs_types.system)
    }
}

impl SystemSet {
    pub fn create(
        ctx: &mut Context,
        name: &str,
        set: SystemSet,
    ) -> Result<SystemSetHandle, ResourceError> {
        ctx.resource
            .create(Some(name), ctx.ecs_types.system_set, set)
            .map(SystemSetHandle)
    }

    pub fn add(ctx: &mut Context, set: SystemSetHandle) {
        ctx.commands.push(ECSCommand::AddSystemSet(set));
    }

    pub fn remove(ctx: &mut Context, set: SystemSetHandle) {
        ctx.commands.push(ECSCommand::RemoveSystemSet(set));
    }

    pub fn enable(ctx: &mut Context, set: SystemSetHandle, enabled: bool) {}
}

impl SystemStage {
    pub fn invoke(ctx: &mut Context, stage: SystemStageHandle, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<SystemStageHandle> {
        ctx.resource.find_typed(name, ctx.ecs_types.system_stage)
    }
}

impl ComponentType {
    pub fn create(
        ctx: &mut Context,
        name: &str,
        ty: ComponentType,
    ) -> Result<ComponentTypeHandle, ResourceError> {
        ctx.resource
            .create(Some(name), ctx.ecs_types.component, ty)
            .map(ComponentTypeHandle)
    }

    pub fn find(ctx: &Context, name: impl ToUID) -> Option<ComponentTypeHandle> {
        ctx.resource.find_typed(name, ctx.ecs_types.component)
    }
}
