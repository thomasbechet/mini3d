#![no_std]

use core::cell::RefCell;

use alloc::{boxed::Box, collections::VecDeque, rc::Rc, vec::Vec};
use api::API;
use component::{event::UserEvent, hierarchy::Hierarchy, input::{InputAction, InputAxis, InputText}, texture::Texture, transform::Transform};
use db::entity::Entity;
use event::EventTable;
use logger::{level::LogLevel, LoggerManager};
use mini3d_db::database::Database;
use mini3d_derive::Error;
use mini3d_input::{provider::InputProvider, InputManager};
use mini3d_io::{disk::DiskManager, provider::DiskProvider};
use mini3d_logger::provider::LoggerProvider;
use mini3d_renderer::{provider::RendererProvider, RendererManager};
use mini3d_scheduler::{Scheduler, StageId, SystemHandle, SystemState};
use mini3d_serialize::{Decoder, DecoderError, Encoder, EncoderError};
use mini3d_utils::slotmap::SecondaryMap;
use system::System;

pub mod api;
pub mod component;
pub mod event;
pub mod import;
pub mod system;

extern crate alloc;

#[cfg(test)]
extern crate std;

pub use crate as mini3d_runtime;
pub use mini3d_db as db;
pub use mini3d_logger as logger;
pub use mini3d_math as math;
pub use mini3d_renderer as renderer;
pub use mini3d_utils as utils;

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
pub(crate) struct RuntimeState {
    pub(crate) next_tick_stages: VecDeque<StageId>,
    pub(crate) next_stages: VecDeque<StageId>,
    request_stop: bool,
    target_tps: u16,
    event_entity: Entity,
    pub(crate) events: EventTable,
    pub(crate) native_systems: Rc<RefCell<SecondaryMap<SystemHandle, Option<Box<dyn System>>>>>,
    created_native_systems: Vec<(SystemHandle, Box<dyn System>)>,
    rebuild_scheduler: bool,
}

pub struct Runtime {
    pub(crate) scheduler: Scheduler,
    pub(crate) database: Database,
    pub(crate) disk: DiskManager,
    pub(crate) input: InputManager,
    pub(crate) renderer: RendererManager,
    pub(crate) logger: LoggerManager,
    pub(crate) state: RuntimeState,
}

pub(crate) fn execute_stage(stage: StageId, api: &mut API) {
    // Acquire first node of this stage
    let mut next_node = api.scheduler.first_node(stage);
    // Iterate over stage nodes
    while next_node.is_some() {
        let node = next_node.unwrap();
        // Execute node
        let systems = api.scheduler.systems(node);
        if systems.len() == 1 {
            // Acquire system table (read only)
            let native_systems = api.state.native_systems.clone();
            // Run the callback
            native_systems
                .as_ref()
                .borrow()
                .get(systems[0])
                .unwrap()
                .as_ref()
                .unwrap()
                .run(api);
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
            database: Default::default(),
            disk: Default::default(),
            input: Default::default(),
            renderer: Default::default(),
            logger: Default::default(),
            state: Default::default(),
        };
        runtime.state.target_tps = config.target_tps;
        runtime.state.events.setup(&mut runtime.scheduler);
        runtime.scheduler.rebuild();
        let api = &mut API {
            database: &mut runtime.database,
            scheduler: &mut runtime.scheduler,
            logger: &mut runtime.logger,
            state: &mut runtime.state,
            input: &mut runtime.input,
            renderer: &mut runtime.renderer,
        };
        Texture::register(api);
        Transform::register(api);
        Hierarchy::register(api);
        UserEvent::register(api);
        InputAction::register(api);
        InputAxis::register(api);
        InputText::register(api);
        if let Some(bootstrap) = config.bootstrap {
            bootstrap(api);
        }
        runtime
            .state
            .next_tick_stages
            .push_back(runtime.state.events.start.unwrap());
        runtime.flush_database_and_scheduler();
        runtime
    }

    fn flush_database_and_scheduler(&mut self) {
        // Update scheduler if needed
        if self.state.rebuild_scheduler {
            self.database.rebuild();

            for id in self.scheduler.systems_from_state(SystemState::Created) {
                let found = self
                    .state
                    .created_native_systems
                    .iter()
                    .position(|(sid, _)| *sid == id)
                    .unwrap();
                let native_system = self.state.created_native_systems.swap_remove(found);
                self.state
                    .native_systems
                    .borrow_mut()
                    .insert(id, Some(native_system.1));
                self.logger
                    .log(format_args!("found {}", found), LogLevel::Debug, None);
            }

            assert!(self.state.created_native_systems.is_empty());

            self.scheduler.rebuild();

            for id in self.scheduler.systems_from_state(SystemState::Running) {
                // Thank you rust
                self.state
                    .native_systems
                    .borrow_mut()
                    .get_mut(id)
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .resolve(&self.database);
            }

            self.state.rebuild_scheduler = false;
        }
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
        self.state.next_stages.clear();
        for stage in self.state.next_tick_stages.drain(..) {
            self.state.next_stages.push_back(stage);
        }
        // Append tick stage
        self.state
            .next_stages
            .push_back(self.state.events.tick.unwrap());
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
        while let Some(stage) = self.state.next_stages.pop_front() {
            execute_stage(
                stage,
                &mut API {
                    database: &mut self.database,
                    scheduler: &mut self.scheduler,
                    logger: &mut self.logger,
                    state: &mut self.state,
                    input: &mut self.input,
                    renderer: &mut self.renderer,
                },
            );
        }

        // ================= POST-UPDATE STAGE ================== //

        self.flush_database_and_scheduler();

        Ok(())
    }
}
