use crate::{uid::UID, registry::RegistryManager, ecs::{world::World, scheduler::Scheduler}, input::InputManager, renderer::RendererManager, asset::AssetManager};
use core::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use self::{asset::AssetContext, renderer::RendererContext, input::InputContext, world::WorldManagerContext, scheduler::SchedulerContext, procedure::ProcedureContext, time::TimeContext};

pub mod asset;
pub mod input;
pub mod renderer;
pub mod scheduler;
pub mod procedure;
pub mod time;
pub mod world;

pub struct SystemContext<'a> {

    // Managers
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) asset: &'a RefCell<AssetManager>,
    pub(crate) input: &'a RefCell<InputManager>,
    pub(crate) renderer: &'a RefCell<RendererManager>,
    
    // Scheduler
    pub(crate) scheduler: &'a RefCell<Scheduler>,

    // Worlds
    pub(crate) worlds: &'a RefCell<HashMap<UID, RefCell<Box<World>>>>,
    pub(crate) active_world: UID,
    pub(crate) change_world: &'a mut Option<UID>,

    // Procedures
    pub(crate) active_procedure: UID,
    pub(crate) frame_procedures: &'a mut VecDeque<UID>,
    pub(crate) next_frame_procedures: &'a mut VecDeque<UID>,

    // Time
    pub(crate) delta_time: f64,
    pub(crate) time: f64,
}

impl<'a> SystemContext<'a> {

    pub fn asset(&self) -> AssetContext<'_> {
        AssetContext::new(self.registry, &mut self.asset.borrow_mut())
    }

    pub fn renderer(&self) -> RendererContext<'_> {
        RendererContext::new(self.renderer)
    }

    pub fn input(&self) -> InputContext<'_> {
        InputContext::new(&self.input.borrow())
    }

    pub fn world(&self) -> WorldManagerContext<'_> {
        WorldManagerContext::new(self.registry, self.worlds, self.active_world, self.change_world)
    }

    pub fn scheduler(&self) -> SchedulerContext<'_> {
        SchedulerContext::new(&mut self.scheduler.borrow_mut())
    }

    pub fn procedure(&self) -> ProcedureContext<'_> {
        ProcedureContext::new(self.active_procedure, self.frame_procedures, self.next_frame_procedures)
    }

    pub fn time(&self) -> TimeContext {
        TimeContext::new(self.time, self.delta_time)
    }
}