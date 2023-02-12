use crate::{uid::UID, registry::RegistryManager, scene::world::World, input::InputManager, renderer::RendererManager};
use core::cell::RefCell;
use self::{asset::AssetContext, renderer::RendererContext, scene::SceneContext, world::WorldContext, input::InputContext};

pub mod asset;
pub mod input;
pub mod renderer;
pub mod scene;
pub mod world;

pub struct SystemContext<'a> {

    registry: &'a RefCell<RegistryManager>,
    world: &'a RefCell<World>,
    input: &'a RefCell<InputManager>,
    renderer: &'a RefCell<RendererManager>,

    delta_time: f64,
    time: f64,
    active_scene: UID,
}

impl<'a> SystemContext<'a> {

    pub fn asset(&self) -> AssetContext<'_> {

    }

    pub fn renderer(&self) -> RendererContext<'_> {
        RendererContext::new(self.renderer)
    }

    pub fn scene(&self) -> SceneContext<'_> {

    }

    pub fn world(&self) -> WorldContext<'_> {
        WorldContext::new(self.registry, self.world)
    }

    pub fn input(&self) -> InputContext<'_> {
        InputContext::new(self.input.borrow().into())
    }

    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn active_scene(&self) -> UID {
        self.active_scene
    }
}