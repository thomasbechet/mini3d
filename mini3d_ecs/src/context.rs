use core::any::Any;

use alloc::vec::Vec;

use crate::{entity::Entity, scheduler::Invocation};

pub(crate) enum SystemCommand {
    Despawn(Entity),
    EnableComponentType(Entity),
    DisableComponentType(Entity),
    EnableSystem(Entity),
    DisableSystem(Entity),
    EnableSystemStage(Entity),
    DisableSystemStage(Entity),
}

pub struct Context<'a> {
    pub(crate) user: &'a mut dyn Any,
    pub(crate) commands: &'a mut Vec<SystemCommand>,
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

    pub fn spawn(&mut self) -> Entity {
        Entity::null()
    }

    pub fn dump_entity(&mut self, entity: Entity, path: &str) {}
}

pub struct Command;

impl Command {
    pub fn load(ctx: &mut Context) {}

    pub fn despawn(ctx: &mut Context, entity: Entity) {
        ctx.commands.push(SystemCommand::Despawn(entity));
    }

    pub fn enable_component_type(ctx: &mut Context, entity: Entity) {
        ctx.commands
            .push(SystemCommand::EnableComponentType(entity));
    }

    pub fn disable_component_type(ctx: &mut Context, entity: Entity) {
        ctx.commands
            .push(SystemCommand::DisableComponentType(entity));
    }

    pub fn enable_system(ctx: &mut Context, system: Entity) {
        ctx.commands.push(SystemCommand::EnableSystem(system));
    }

    pub fn disable_system(ctx: &mut Context, system: Entity) {
        ctx.commands.push(SystemCommand::DisableSystem(system));
    }

    pub fn enable_system_stage(ctx: &mut Context, stage: Entity) {
        ctx.commands.push(SystemCommand::EnableSystemStage(stage));
    }

    pub fn disable_system_stage(ctx: &mut Context, stage: Entity) {
        ctx.commands.push(SystemCommand::DisableSystemStage(stage));
    }

    pub fn invoke(ctx: &mut Context, stage: Entity, invocation: Invocation) {}
}
