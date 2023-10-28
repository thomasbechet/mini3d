use crate::{
    ecs::{
        container::ContainerTable, entity::EntityTable, query::QueryTable, scheduler::Scheduler,
    },
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
    resource::ResourceManager,
};

use self::{activity::ActivityContext, time::TimeAPI};

pub mod activity;
pub mod ecs;
pub mod input;
pub mod logger;
pub mod renderer;
pub mod resource;
pub mod runtime;
pub mod time;

pub struct Context<'a> {
    pub(crate) activity: &'a mut ActivityContext,
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) runtime: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) cycle: u32,
}
