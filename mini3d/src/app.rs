use anyhow::{Context, Result};
use slotmap::Key;

use crate::asset::AssetManager;
use crate::backend::{BackendDescriptor, Backend, DefaultBackend};
use crate::event::AppEvents;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::program::{ProgramManager, Program, ProgramBuilder, ProgramId};
use crate::request::AppRequests;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct App {
    pub(crate) asset_manager: AssetManager,
    pub(crate) input_manager: InputManager,
    pub(crate) program_manager: ProgramManager,

    default_backend: DefaultBackend,

    accumulator: f64,
}

impl App {

    pub fn new<P: Program + ProgramBuilder + 'static>(data: P::BuildData) -> Result<Self> {
        // Default application state
        let mut app = Self {
            asset_manager: Default::default(), 
            input_manager: Default::default(), 
            program_manager: Default::default(),
            default_backend: Default::default(), 
            accumulator: 0.0,
        };
        // Start initial program
        app.program_manager.run::<P>("root", data, ProgramId::null())?;
        // Return application
        Ok(app)
    }

    pub fn asset(&self) -> &'_ AssetManager {
        &self.asset_manager
    }

    pub fn progress<'a>(
        &'a mut self, 
        backend_descriptor: BackendDescriptor<'a>, 
        events: &AppEvents,
        requests: &mut AppRequests,
        mut delta_time: f64,
    ) -> Result<()> {

        // Build the backend
        let mut backend = Backend::build(backend_descriptor, &mut self.default_backend);

        // ================= DISPATCH STEP ================= //

        // Prepare input manager
        self.input_manager.prepare_dispatch();
        // Dispatch input events
        for event in &events.input {
            self.input_manager.dispatch_event(event);
        }

        // Dispatch system events
        for event in &events.system {
            match event {
                SystemEvent::Shutdown => {
                    requests.shutdown = true;
                },
            }
        }

        // TODO: dispatch more events ...

        // ================= UPDATE STEP ================= //

        // Compute accumulated time since last progress
        if delta_time > MAXIMUM_TIMESTEP {
            delta_time = MAXIMUM_TIMESTEP; // Slowing down
        }
        self.accumulator += delta_time;

        // Prepare resources for drawing
        backend.renderer.reset_command_buffers();

        // Update programs
        self.program_manager.update(
            &mut self.asset_manager, 
            &mut self.input_manager, 
            &mut backend,
            events,
            delta_time,
        ).context("Failed to update program manager")?;

        // ================= FIXED UPDATE STEP ================= //

        delta_time = FIXED_TIMESTEP;
        while self.accumulator >= FIXED_TIMESTEP {
            self.accumulator -= FIXED_TIMESTEP;

            // Process fixed update
        }

        // ================= REQUESTS STEP ================= //

        // Check input requests
        if self.input_manager.reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input_manager.reload_input_mapping = false;
        }

        Ok(())
    }
}