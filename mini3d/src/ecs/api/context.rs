use crate::{
    input::InputManager, logger::LoggerManager, platform::PlatformManager,
    registry::RegistryManager, renderer::RendererManager, resource::ResourceManager,
};

use super::time::TimeAPI;

pub struct Context<'a> {
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) registry: &'a mut RegistryManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) runtime: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
}
