use crate::{
    input::InputManager, logger::LoggerManager, platform::PlatformManager,
    renderer::RendererManager,
};

use self::{ecs::ECSContext, time::TimeContext};

pub mod ecs;
pub mod logger;
pub mod platform;
pub mod renderer;
pub mod time;

pub use logger::*;
pub use platform::*;
pub use renderer::*;
pub use time::*;

pub struct Context<'a> {
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) platform: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeContext,
    pub(crate) ecs: ECSContext<'a>,
}
