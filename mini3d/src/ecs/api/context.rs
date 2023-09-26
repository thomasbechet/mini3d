use crate::{
    asset::AssetManager, input::InputManager, logger::LoggerManager, registry::RegistryManager,
    renderer::RendererManager, system::SystemManager,
};

use super::time::TimeAPI;

pub struct Context<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub registry: &'a mut RegistryManager,
    pub renderer: &'a mut RendererManager,
    pub system: &'a mut SystemManager,
    pub logger: &'a mut LoggerManager,
    pub time: TimeAPI,
}
