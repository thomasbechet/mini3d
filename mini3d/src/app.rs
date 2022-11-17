use anyhow::Result;

use crate::asset::AssetManager;
use crate::backend::{BackendDescriptor, Backend, DefaultBackend};
use crate::ecs::ECSManager;
use crate::event::AppEvents;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::process::{ProcessManager, Process, ProcessBuilder, ProcessManagerContext};
use crate::request::AppRequests;
use crate::script::ScriptManager;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct App {
    pub(crate) asset_manager: AssetManager,
    pub(crate) input_manager: InputManager,
    pub(crate) process_manager: ProcessManager,
    pub(crate) script_manager: ScriptManager,
    pub(crate) ecs_manager: ECSManager,

    default_backend: DefaultBackend,

    accumulator: f64,
}

impl App {

    pub fn new<P: Process + ProcessBuilder + 'static>(data: P::BuildData) -> Result<Self> {
        Ok(Self {
            asset_manager: Default::default(), 
            input_manager: Default::default(), 
            process_manager: ProcessManager::with_root::<P>(data),
            script_manager: Default::default(),
            ecs_manager: Default::default(),
            default_backend: Default::default(),
            accumulator: 0.0,
        })
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
        let backend = Backend::build(backend_descriptor, &mut self.default_backend);

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

        // Update processes
        let mut ctx = ProcessManagerContext {
            asset: &mut self.asset_manager,
            input: &mut self.input_manager,
            script: &mut self.script_manager,
            ecs: &mut self.ecs_manager,
            renderer: backend.renderer,
            events,
            delta_time,
        };
        self.process_manager.update(&mut ctx)?;

        // ================= FIXED UPDATE STEP ================= //

        // delta_time = FIXED_TIMESTEP;
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