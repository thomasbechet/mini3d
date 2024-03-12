#![no_std]

use alloc::{boxed::Box, collections::VecDeque};
use api::API;
use event::ComponentEventStages;
use mini3d_db::database::{ComponentId, Database};
use mini3d_derive::Error;
use mini3d_input::{provider::InputProvider, InputManager};
use mini3d_io::{disk::DiskManager, provider::DiskProvider};
use mini3d_logger::provider::LoggerProvider;
use mini3d_renderer::{provider::RendererProvider, RendererManager};
use mini3d_scheduler::{Scheduler, StageId, SystemId};
use mini3d_serialize::{Decoder, DecoderError, Encoder, EncoderError};
use mini3d_utils::slotmap::SecondaryMap;

pub mod api;
pub mod component;
pub mod event;
pub mod import;

extern crate alloc;

#[cfg(test)]
extern crate std;

pub use mini3d_db::*;
pub use mini3d_logger::*;
pub use mini3d_math::*;
pub use crate as mini3d_runtime;

#[derive(Error, Debug)]
pub enum TickError {
    #[error("Core error")]
    Core,
    #[error("ECS system error")]
    System,
}

#[derive(Clone)]
pub struct RuntimeConfig {
    bootstrap: Option<fn(&mut API)>,
    target_tps: u16,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            bootstrap: None,
            target_tps: 60,
        }
    }
}

impl RuntimeConfig {
    pub fn bootstrap(mut self, bootstrap: fn(&mut API)) -> Self {
        self.bootstrap = Some(bootstrap);
        self
    }
}

pub enum Invocation {
    Immediate,
    NextStage,
    NextTick,
}

#[derive(Default)]
pub(crate) struct Stages {
    pub(crate) next_tick_stages: VecDeque<StageId>,
    pub(crate) next_stages: VecDeque<StageId>,
    pub(crate) start_stage: Option<StageId>,
    pub(crate) tick_stage: Option<StageId>,
    pub(crate) components: SecondaryMap<ComponentId, ComponentEventStages>,
}

#[derive(Default)]
pub(crate) struct RuntimeState {
    request_stop: bool,
    target_tps: u16,
    pub(crate) systems: SecondaryMap<SystemId, Option<fn(&mut API)>>,
    pub(crate) stages: Stages,
}

pub struct Runtime {
    pub(crate) scheduler: Scheduler,
    pub(crate) db: Database,
    pub(crate) disk: DiskManager,
    pub(crate) input: InputManager,
    pub(crate) renderer: RendererManager,
    pub(crate) logger: LoggerManager,
    pub(crate) state: RuntimeState,
}

pub(crate) fn execute_stage(stage: StageId, api: &mut API) {
    debug!(api, "running stage {}", api.scheduler.stage(stage).unwrap().name);
    // Acquire first node of this stage
    let mut next_node = api.scheduler.first_node(stage);
    // Iterate over stage nodes
    while next_node.is_some() {
        let node = next_node.unwrap();
        // Execute node
        let systems = api.scheduler.systems(node);
        if systems.len() == 1 {
            // Find callback
            let callback = &api.state.systems[systems[0]].unwrap();
            // Run the callback
            callback(api);
        } else {
            // TODO: use thread pool
        }
        next_node = api.scheduler.next_node(node);
    }
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Self {
        let mut runtime = Self {
            scheduler: Default::default(),
            db: Default::default(),
            disk: Default::default(),
            input: Default::default(),
            renderer: Default::default(),
            logger: Default::default(),
            state: Default::default(),
        };
        runtime.state.target_tps = config.target_tps;
        runtime.state.stages.start_stage = Some(runtime.scheduler.add_stage("_start").unwrap());
        runtime.state.stages.tick_stage = Some(runtime.scheduler.add_stage("_tick").unwrap());
        runtime.scheduler.rebuild();
        if let Some(bootstrap) = config.bootstrap {
            bootstrap(&mut API {
                db: &mut runtime.db,
                scheduler: &mut runtime.scheduler,
                logger: &mut runtime.logger,
                state: &mut runtime.state,
                input: &mut runtime.input,
            });
        }
        runtime
            .state
            .stages
            .next_tick_stages
            .push_back(runtime.state.stages.start_stage.unwrap());
        runtime
    }

    pub fn set_renderer(&mut self, provider: impl RendererProvider + 'static) {
        self.renderer.set_provider(Box::new(provider));
    }

    pub fn set_input(&mut self, provider: impl InputProvider + 'static) {
        self.input.set_provider(Box::new(provider));
    }

    pub fn set_disk(&mut self, provider: impl DiskProvider + 'static) {
        self.disk.set_provider(Box::new(provider));
    }

    pub fn set_logger(&mut self, provider: impl LoggerProvider + 'static) {
        self.logger.set_provider(Box::new(provider));
    }

    pub fn save(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub fn load(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        Ok(())
    }

    pub fn target_tps(&self) -> u16 {
        self.state.target_tps
    }

    fn prepare_next_stages(&mut self) {
        // Collect previous frame stages
        self.state.stages.next_stages.clear();
        for stage in self.state.stages.next_tick_stages.drain(..) {
            self.state.stages.next_stages.push_back(stage);
        }
        // Append tick stage
        self.state
            .stages
            .next_stages
            .push_back(self.state.stages.tick_stage.unwrap());
    }

    pub fn tick(&mut self) -> Result<(), TickError> {
        // ================= PREPARE STAGE ================== //

        // ================= DISPATCH EVENTS STAGE ================= //

        // Prepare input manager
        self.input.prepare_dispatch();
        // Dispatch input events
        self.input.dispatch_events();

        // Dispatch network events

        // Dispatch storage events

        // TODO: dispatch more events ...

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        // Prepare frame stages
        self.prepare_next_stages();

        // Run stages
        // TODO: protect against infinite loops
        while let Some(stage) = self.state.stages.next_stages.pop_front() {
            execute_stage(
                stage,
                &mut API {
                    db: &mut self.db,
                    scheduler: &mut self.scheduler,
                    logger: &mut self.logger,
                    state: &mut self.state,
                    input: &mut self.input,
                },
            );
        }

        // ================= POST-UPDATE STAGE ================== //

        Ok(())
    }
}
