use crate::{
    activity::ActivityManager,
    ecs::{ECSHandles, ECSInstance},
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
    resource::ResourceManager,
};

use self::time::TimeAPI;

pub mod activity;
pub mod ecs;
pub mod input;
pub mod logger;
pub mod platform;
pub mod renderer;
pub mod resource;
pub mod time;

pub struct Context<'a> {
    pub(crate) activity: &'a mut ActivityManager,
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) platform: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
    pub(crate) ecs: &'a mut ECSInstance,
    pub(crate) ecs_types: &'a ECSHandles,
}
