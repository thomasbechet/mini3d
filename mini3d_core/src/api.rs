use alloc::vec::Vec;

use crate::{
    ecs::{
        entity::{Entity, EntityTable},
        scheduler::Scheduler,
        ECSCommand, ECSHandles,
    },
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
    resource::ResourceManager,
};

use self::time::TimeAPI;

pub mod ecs;
pub mod input;
pub mod logger;
pub mod platform;
pub mod renderer;
pub mod resource;
pub mod time;

pub struct Context<'a> {
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) entity_created: &'a mut Vec<Entity>,
    pub(crate) entity_destroyed: &'a mut Vec<Entity>,
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) platform: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
    pub(crate) ecs_types: &'a ECSHandles,
    pub(crate) commands: &'a mut Vec<ECSCommand>,
}
