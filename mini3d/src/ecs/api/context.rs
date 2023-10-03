use crate::{
    asset::AssetManager, input::InputManager, logger::LoggerManager, platform::PlatformManager,
    registry::RegistryManager, renderer::RendererManager,
};

use super::time::TimeAPI;

pub struct Context<'a> {
    pub(crate) asset: &'a mut AssetManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) registry: &'a mut RegistryManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) runtime: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
}
