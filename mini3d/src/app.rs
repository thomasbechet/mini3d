use anyhow::{Context, Result};
use slotmap::Key;

use crate::asset::AssetManager;
use crate::backend::{BackendDescriptor, Backend, DefaultBackend};
use crate::event::AppEvents;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::program::{ProgramManager, Program, ProgramBuilder, ProgramId};
use crate::request::AppRequests;

pub struct App {
    pub(crate) asset_manager: AssetManager,
    pub(crate) input_manager: InputManager,
    pub(crate) program_manager: ProgramManager,

    default_backend: DefaultBackend,
}

impl App {

    pub fn new<P: Program + ProgramBuilder + 'static>(data: P::BuildData) -> Result<Self> {
        // Default application state
        let mut app = Self {
            asset_manager: Default::default(), 
            input_manager: Default::default(), 
            program_manager: Default::default(), 
            default_backend: Default::default(), 
        };
        // Start initial program
        app.program_manager.run::<P>("root", data, ProgramId::null())?;
        // Return application
        Ok(app)
    }

    pub fn progress<'a>(
        &'a mut self, 
        backend_descriptor: BackendDescriptor<'a>, 
        events: &mut AppEvents,
        requests: &mut AppRequests,
        delta_time: f32,
    ) -> Result<()> {
    
        // Build the backend
        let mut backend = Backend::build(backend_descriptor, &mut self.default_backend);

        // Dispatch import asset events
        for event in events.assets.drain(..) {
            self.asset_manager.dispatch_event(event)?;
        }

        // Prepare input manager
        self.input_manager.prepare_dispatch();
        // Dispatch input events
        for event in events.inputs.drain(..) {
            self.input_manager.dispatch_event(&event);
        }

        // Dispatch system events
        for event in events.systems.drain(..) {
            match event {
                SystemEvent::Shutdown => {
                    requests.shutdown = true;
                },
            }
        }

        // TODO: dispatch more events ...

        // Ensure all events have been dispatched
        events.clear();

        // Prepare resources for drawing
        backend.renderer.reset_command_buffers();

        // Update programs
        self.program_manager.update(
            &mut self.asset_manager, 
            &mut self.input_manager, 
            &mut backend,
            delta_time,
        ).context("Failed to update program manager")?;

        // Check input requests
        if self.input_manager.reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input_manager.reload_input_mapping = false;
        }

        Ok(())
    }
}