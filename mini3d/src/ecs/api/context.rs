use crate::{
    asset::AssetManager, input::InputManager, logger::LoggerManager, registry::RegistryManager,
    renderer::RendererManager, system::SystemManager,
};

use super::time::TimeAPI;

pub struct Context<'a> {
    pub(crate) asset: &'a mut AssetManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) registry: &'a mut RegistryManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) system: &'a mut SystemManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) time: TimeAPI,
}
