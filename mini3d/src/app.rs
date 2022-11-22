use anyhow::Result;

use crate::asset::AssetManager;
use crate::backend::{BackendDescriptor, Backend, DefaultBackend};
use crate::feature::{asset, component, system, signal, process};
use crate::ecs::ECSManager;
use crate::event::AppEvents;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::process::{ProcessManager, ProcessManagerContext};
use crate::request::AppRequests;
use crate::script::ScriptManager;
use crate::signal::SignalManager;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct App {
    pub asset: AssetManager,
    pub input: InputManager,
    pub process: ProcessManager,
    pub script: ScriptManager,
    pub ecs: ECSManager,
    pub signal: SignalManager,

    default_backend: DefaultBackend,

    accumulator: f64,
}

impl App {

    fn register_feature(&mut self) -> Result<()> {

        // Assets
        self.asset.register::<asset::font::Font>("font")?;
        self.asset.register::<asset::input_action::InputAction>("input_action")?;
        self.asset.register::<asset::input_axis::InputAxis>("input_axis")?;
        self.asset.register::<asset::input_table::InputTable>("input_table")?;
        self.asset.register::<asset::material::Material>("material")?;
        self.asset.register::<asset::mesh::Mesh>("mesh")?;
        self.asset.register::<asset::model::Model>("model")?;
        self.asset.register::<asset::rhai_script::RhaiScript>("rhai_script")?;
        self.asset.register::<asset::system_schedule::SystemSchedule>("system_schedule")?;
        self.asset.register::<asset::texture::Texture>("texture")?;

        // Components
        self.ecs.register_component::<component::camera::CameraComponent>("camera")?;
        self.ecs.register_component::<component::free_fly::FreeFlyComponent>("free_fly")?;
        self.ecs.register_component::<component::lifecycle::LifecycleComponent>("lifecycle")?;
        self.ecs.register_component::<component::model::ModelComponent>("model")?;
        self.ecs.register_component::<component::rhai_scripts::RhaiScriptsComponent>("rhai_scripts")?;
        self.ecs.register_component::<component::rotator::RotatorComponent>("rotator")?;
        self.ecs.register_component::<component::script_storage::ScriptStorageComponent>("script_storage")?;
        self.ecs.register_component::<component::transform::TransformComponent>("transform")?;

        // Processes
        self.process.register::<process::profiler::ProfilerProcess>("profiler")?;

        // Systems
        self.ecs.register_system("despawn_entities", system::despawn::run)?;
        self.ecs.register_system("free_fly", system::free_fly::run)?;
        self.ecs.register_system("renderer_check_lifecycle", system::renderer::check_lifecycle)?;
        self.ecs.register_system("renderer_transfer_transforms", system::renderer::transfer_transforms)?;
        self.ecs.register_system("renderer_update_camera", system::renderer::update_camera)?;
        self.ecs.register_system("rhai_update_scripts", system::rhai::update_scripts)?;
        self.ecs.register_system("rotator", system::rotator::run)?;

        // Signals
        self.signal.register::<signal::command::CommandSignal>("command")?;

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let mut app = Self {
            asset: Default::default(), 
            input: Default::default(), 
            process: Default::default(),
            script: Default::default(),
            ecs: Default::default(),
            signal: Default::default(),
            default_backend: Default::default(),
            accumulator: 0.0,
        };
        app.register_feature()?;
        Ok(app)
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
        self.input.prepare_dispatch();
        // Dispatch input events
        for event in &events.input {
            self.input.dispatch_event(event);
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
            asset: &mut self.asset,
            input: &mut self.input,
            script: &mut self.script,
            ecs: &mut self.ecs,
            signal: &mut self.signal,
            renderer: backend.renderer,
            events,
            delta_time,
        };
        self.process.update(&mut ctx)?;

        // ================= FIXED UPDATE STEP ================= //

        // delta_time = FIXED_TIMESTEP;
        while self.accumulator >= FIXED_TIMESTEP {
            self.accumulator -= FIXED_TIMESTEP;

            // Process fixed update
        }

        // ================= REQUESTS STEP ================= //

        // Check input requests
        if self.input.reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input.reload_input_mapping = false;
        }

        // ================= CLEANUP STEP ================= // 
        self.signal.cleanup();

        Ok(())
    }
}