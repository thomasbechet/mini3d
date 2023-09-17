use crate::{
    asset::AssetManager, input::InputManager, logger::LoggerManager, registry::RegistryManager,
    renderer::RendererManager, system::SystemManager,
};

use self::time::TimeAPI;

pub mod ecs;
pub mod logger;
pub mod time;

pub struct ExclusiveAPI<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub registry: &'a mut RegistryManager,
    pub renderer: &'a mut RendererManager,
    pub system: &'a mut SystemManager,
    pub logger: &'a mut LoggerManager,
    pub time: TimeAPI,
}

pub struct ParallelAPI<'a> {
    pub asset: &'a AssetManager,
    pub input: &'a InputManager,
    pub registry: &'a RegistryManager,
    pub renderer: &'a RendererManager,
    pub system: &'a SystemManager,
    pub logger: &'a LoggerManager,
    pub time: TimeAPI,
}
