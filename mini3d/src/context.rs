use crate::{uid::UID, registry::RegistryManager, scene::{world::World, SceneInfo}, input::InputManager, renderer::RendererManager, asset::AssetManager};
use core::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use self::{asset::AssetContext, renderer::RendererContext, scene::{SceneContext, SceneCommand}, world::WorldContext, input::InputContext, schedule::ScheduleContext};

pub mod asset;
pub mod input;
pub mod renderer;
pub mod scene;
pub mod schedule;
pub mod world;

pub struct SystemContext<'a> {

    pub(crate) registry: &'a RefCell<RegistryManager>,
    asset: &'a RefCell<AssetManager>,
    input: &'a RefCell<InputManager>,
    renderer: &'a RefCell<RendererManager>,
    
    world: &'a mut World,
    delta_time: f64,
    time: f64,
    active_scene: UID,

    scene_info: &'a HashMap<UID, SceneInfo>,
    scene_commands: &'a mut Vec<SceneCommand>,
    signal_queue: &'a mut VecDeque<UID>,
}

impl<'a> SystemContext<'a> {

    pub(crate) fn new(
        registry: &'a RefCell<RegistryManager>,
        asset: &'a RefCell<AssetManager>,
        input: &'a RefCell<InputManager>,
        renderer: &'a RefCell<RendererManager>,
        world: &'a mut World,
        delta_time: f64,
        time: f64,
        active_scene: UID,
        scene_info: &'a HashMap<UID, SceneInfo>,
        scene_commands: &'a mut Vec<SceneCommand>,
        signal_queue: &'a mut VecDeque<UID>,
    ) -> Self {
        Self {
            registry,
            asset,
            input,
            renderer,
            world,
            delta_time,
            time,
            active_scene,
            scene_info,
            scene_commands,
            signal_queue,
        }
    }

    pub fn asset(&self) -> AssetContext<'_> {
        AssetContext::new(self.registry, self.asset)
    }

    pub fn renderer(&self) -> RendererContext<'_> {
        RendererContext::new(self.renderer)
    }

    pub fn scene(&self) -> SceneContext<'_> {
        SceneContext::new(self.scene_info, self.scene_commands)
    }

    pub fn schedule(&self) -> ScheduleContext<'_> {
        ScheduleContext::new(self.signal_queue)
    }

    pub fn world(&self) -> WorldContext<'_> {
        WorldContext::new(self.registry, &mut self.world)
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