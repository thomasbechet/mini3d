use alloc::vec::Vec;

use crate::{
    ecs::{
        entity::{Entity, EntityTable},
        scheduler::Scheduler,
        ECSCommand, ECSViews,
    },
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
};

use self::time::TimeAPI;

pub mod logger;
pub mod platform;
pub mod renderer;
pub mod time;

pub use logger::*;
pub use platform::*;
pub use renderer::*;
pub use time::*;

pub struct Context<'a> {
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) entity_created: &'a mut Vec<Entity>,
    pub(crate) entity_destroyed: &'a mut Vec<Entity>,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) platform: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
    pub(crate) ecs_types: &'a ECSViews,
    pub(crate) commands: &'a mut Vec<ECSCommand>,
}
