#![no_std]

use alloc::{boxed::Box, vec::Vec};
use api::API;
use mini3d_db::database::Database;
use mini3d_derive::Error;
use mini3d_input::{provider::InputProvider, InputManager};
use mini3d_io::{disk::DiskManager, provider::DiskProvider};
use mini3d_logger::{provider::LoggerProvider, LoggerManager};
use mini3d_renderer::{provider::RendererProvider, RendererManager};
use mini3d_scheduler::{Scheduler, SystemId};
use mini3d_serialize::{Decoder, DecoderError, Encoder, EncoderError};
use mini3d_utils::slotmap::SecondaryMap;

pub mod api;
pub mod import;

extern crate alloc;

#[cfg(test)]
extern crate std;

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
    renderer: bool,
    target_tps: u16,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            bootstrap: None,
            renderer: true,
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

pub struct Runtime {
    pub(crate) scheduler: Scheduler,
    pub(crate) db: Database,
    pub(crate) disk: DiskManager,
    pub(crate) input: InputManager,
    pub(crate) renderer: RendererManager,
    pub(crate) logger: LoggerManager,
    request_stop: bool,
    target_tps: u16,
    pub(crate) callbacks: SecondaryMap<SystemId, Option<fn(&mut API)>>,
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
            request_stop: false,
            target_tps: config.target_tps,
            callbacks: Default::default(),
        };
        if let Some(bootstrap) = config.bootstrap {
            bootstrap(&mut API { db: &mut runtime.db, input: &mut runtime.input });
        }
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
        self.target_tps
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
        self.scheduler.prepare_next_frame_stages();

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let systems = self.scheduler.next_systems();
            if systems.is_none() {
                break;
            }
            let systems = systems.unwrap();

            // Execute node
            if systems.len() == 1 {
                // Find callback
                let callback = &self.callbacks[systems[0]].unwrap();

                // Run the callback
                callback(&mut API {
                    db: &mut self.db,
                    input: &mut self.input,
                });
            } else {
                // TODO: use thread pool
            }
        }

        // ================= POST-UPDATE STAGE ================== //

        Ok(())
    }
}
