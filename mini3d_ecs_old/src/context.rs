use core::any::Any;

use alloc::vec::Vec;

use crate::{
    component::{component_type::ComponentType, system::System, system_stage::SystemStage},
    entity::Entity,
    scheduler::Invocation,
};

pub(crate) enum ContextCommand {
    EnableComponentType(Entity),
    DisableComponentType(Entity),
    EnableSystem(Entity),
    DisableSystem(Entity),
    EnableSystemStage(Entity),
    DisableSystemStage(Entity),
    ReflectEntity(Entity, Entity),
}

pub struct Context<'a> {
    pub(crate) user: &'a mut dyn Any,
    pub(crate) commands: &'a mut Vec<ContextCommand>,
}

impl<'a> Context<'a> {
    pub fn user_mut<UserContext: 'static>(&mut self) -> Option<&mut UserContext> {
        self.user.downcast_mut::<UserContext>()
    }

    pub unsafe fn user_mut_unchecked<UserContext: 'static>(&mut self) -> &mut UserContext {
        self.user_mut::<UserContext>().unwrap()
    }

    pub fn user<UserContext: 'static>(&self) -> Option<&UserContext> {
        self.user.downcast_ref::<UserContext>()
    }

    pub unsafe fn user_unchecked<UserContext: 'static>(&self) -> &UserContext {
        self.user::<UserContext>().unwrap()
    }
}

impl Entity {
    pub fn spawn(ctx: &mut Context) -> Self {
        ctx.entities.spawn()
    }

    pub fn despawn(ctx: &mut Context, entity: Entity) {
        ctx.commands.push(ContextCommand::Despawn(entity));
    }
}

impl ComponentType {
    pub fn enable(ctx: &mut Context, entity: Entity) {
        ctx.commands
            .push(ContextCommand::EnableComponentType(entity));
    }

    pub fn disable(ctx: &mut Context, entity: Entity) {
        ctx.commands
            .push(ContextCommand::DisableComponentType(entity));
    }
}

impl System {
    pub fn enable(ctx: &mut Context, system: Entity) {
        ctx.commands.push(ContextCommand::EnableSystem(system));
    }

    pub fn disable(ctx: &mut Context, system: Entity) {
        ctx.commands.push(ContextCommand::DisableSystem(system));
    }
}

impl SystemStage {
    pub fn tick(ctx: &Context) -> Entity {
        ctx.entities.tick_stage
    }

    pub fn bootstrap(ctx: &Context) -> Entity {
        ctx.entities.bootstrap_stage
    }

    pub fn enable(ctx: &mut Context, stage: Entity) {
        ctx.commands.push(ContextCommand::EnableSystemStage(stage));
    }

    pub fn disable(ctx: &mut Context, stage: Entity) {
        ctx.commands.push(ContextCommand::DisableSystemStage(stage));
    }

    pub fn invoke(ctx: &mut Context, stage: Entity, invocation: Invocation) {}
}
