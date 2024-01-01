use alloc::boxed::Box;
use mini3d_derive::{fixed, Error};

use crate::disk::provider::DiskProvider;
use crate::disk::DiskManager;
use crate::ecs::component::ComponentType;
use crate::ecs::context::time::TimeAPI;
use crate::ecs::context::Context;
use crate::ecs::scheduler::Invocation;
use crate::ecs::{self, ECSManager, ECSUpdateContext};
use crate::input::provider::InputProvider;
use crate::input::{self, InputManager};
use crate::logger::provider::LoggerProvider;
use crate::logger::LoggerManager;
use crate::physics::PhysicsManager;
use crate::platform::provider::PlatformProvider;
use crate::platform::PlatformManager;
use crate::renderer::provider::RendererProvider;
use crate::renderer::{self, RendererManager};
use crate::script;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};

#[derive(Error, Debug)]
pub enum TickError {
    #[error("Core error")]
    Core,
    #[error("ECS system error")]
    System,
}

#[derive(Clone)]
pub struct SimulationConfig {
    bootstrap: Option<fn(&mut Context)>,
    renderer: bool,
    target_tps: u16,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            bootstrap: None,
            renderer: true,
            target_tps: 60,
        }
    }
}

impl SimulationConfig {
    pub fn bootstrap(mut self, bootstrap: fn(&mut Context)) -> Self {
        self.bootstrap = Some(bootstrap);
        self
    }
}

pub struct Simulation {
    pub(crate) ecs: ECSManager,
    pub(crate) storage: DiskManager,
    pub(crate) input: InputManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    pub(crate) platform: PlatformManager,
    pub(crate) logger: LoggerManager,
}

impl Simulation {
    fn setup_resource_manager(&mut self) {
        self.resource.define_meta_type();
    }

    fn define_resource_types(&mut self, config: &SimulationConfig) {
        macro_rules! define_resource {
            ($resource: ty) => {
                self.resource
                    .create_resource_type(
                        Some(<$resource>::NAME),
                        ResourceType::native::<$resource>(),
                    )
                    .unwrap()
            };
        }

        self.ecs.views.component = define_resource!(ecs::resource::ComponentType);
        self.ecs.views.system = define_resource!(ecs::resource::System);
        self.ecs.views.system_set = define_resource!(ecs::resource::SystemSet);
        self.ecs.views.system_stage = define_resource!(ecs::resource::SystemStage);

        self.input.views.action = define_resource!(input::component::InputAction);
        self.input.views.axis = define_resource!(input::component::InputAxis);
        self.input.views.text = define_resource!(input::component::InputText);

        self.renderer.handles.font = define_resource!(renderer::resource::Font);
        self.renderer.handles.material = define_resource!(renderer::resource::Material);
        self.renderer.handles.mesh = define_resource!(renderer::resource::Mesh);
        self.renderer.handles.texture = define_resource!(renderer::resource::Texture);
        self.renderer.handles.model = define_resource!(renderer::resource::Model);
        self.renderer.handles.transform = define_resource!(renderer::resource::RenderTransform);

        define_resource!(script::component::structure::StructDefinition);
        define_resource!(script::component::script::Script);
        define_resource!(script::component::program::Program);
    }

    fn define_component_types(&mut self, config: &SimulationConfig) {
        macro_rules! define_component {
            ($component: ty, $storage: expr) => {
                self.resource
                    .create(
                        Some(<$component>::NAME),
                        self.ecs.handles.component,
                        ComponentType::native::<$component>($storage),
                    )
                    .unwrap()
            };
        }

        macro_rules! define_exclusive_system {
            ($system: ty) => {
                self.resource
                    .create(
                        self.ecs.handles.system,
                        Some(<$system>::NAME),
                        System::native_exclusive::<$system>(),
                    )
                    .unwrap()
            };
        }

        macro_rules! define_parallel_system {
            ($system: ty) => {
                self.resource
                    .create(
                        Some(<$system>::NAME),
                        self.ecs.handles.system,
                        System::native_parallel::<$system>(),
                    )
                    .unwrap()
            };
        }

        self.ecs.containers.add_container(ComponentType::native(<$component>::NAME, true))

        define_component!(ecs::component::FreeFly, ComponentStorage::Single);
        define_component!(ecs::component::Rotator, ComponentStorage::Single);
        define_component!(ecs::component::Transform, ComponentStorage::Single);
        define_component!(ecs::component::Hierarchy, ComponentStorage::Single);
        define_component!(ecs::component::LocalToWorld, ComponentStorage::Single);

        define_parallel_system!(ecs::component::FreeFlySystem);
        define_parallel_system!(ecs::component::RotatorSystem);
        define_parallel_system!(ecs::component::PropagateTransforms);

        if config.renderer {
            define_component!(renderer::component::Camera, ComponentStorage::Single);
            define_component!(renderer::component::StaticMesh, ComponentStorage::Single);
            define_component!(renderer::resource::Tilemap, ComponentStorage::Single);
            define_component!(renderer::resource::Tileset, ComponentStorage::Single);
            define_component!(renderer::resource::Viewport, ComponentStorage::Single);
            define_component!(renderer::resource::Canvas, ComponentStorage::Single);
        }

        self.ecs.views.start_stage = self
            .resource
            .create(
                Some(SystemStage::START),
                self.ecs.views.system_stage,
                SystemStage::default(),
            )
            .unwrap()
            .into();
        self.ecs.views.tick_stage = self
            .resource
            .create(
                Some(SystemStage::TICK),
                self.ecs.views.system_stage,
                SystemStage::default(),
            )
            .unwrap()
            .into();
    }

    fn setup_ecs(&mut self, config: &SimulationConfig) {
        self.ecs.target_tps = config.target_tps;
        self.ecs
            .scheduler
            .invoke(self.ecs.views.start_stage, Invocation::Immediate);
    }

    fn run_bootstrap(&mut self, config: &SimulationConfig) {
        if let Some(bootstrap) = config.bootstrap {
            bootstrap(&mut Context {
                entities: &mut self.ecs.entities,
                scheduler: &mut self.ecs.scheduler,
                entity_created: &mut self.ecs.entity_created,
                entity_destroyed: &mut self.ecs.entity_destroyed,
                resource: &mut self.resource,
                input: &mut self.input,
                renderer: &mut self.renderer,
                platform: &mut self.platform,
                logger: &mut self.logger,
                time: TimeAPI {
                    delta: fixed!(0),
                    frame: 0,
                    target_tps: self.ecs.target_tps,
                },
                ecs_types: &self.ecs.views,
                commands: &mut self.ecs.commands,
            });
            self.ecs.flush_commands(&mut self.resource);
        }
    }

    pub fn new(config: SimulationConfig) -> Self {
        let mut simulation = Self {
            ecs: Default::default(),
            storage: Default::default(),
            resource: Default::default(),
            input: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            platform: Default::default(),
            logger: Default::default(),
        };
        simulation.setup_resource_manager();
        simulation.define_resource_types(&config);
        simulation.define_component_types(&config);
        simulation.setup_ecs(&config);
        simulation.run_bootstrap(&config);
        simulation
    }

    pub fn set_renderer(&mut self, provider: impl RendererProvider + 'static) {
        self.renderer.set_provider(Box::new(provider));
    }

    pub fn set_input(&mut self, provider: impl InputProvider + 'static) {
        self.input.set_provider(Box::new(provider));
    }

    pub fn set_platform(&mut self, provider: impl PlatformProvider + 'static) {
        self.platform.set_provider(Box::new(provider));
    }

    pub fn set_storage(&mut self, provider: impl DiskProvider + 'static) {
        self.storage.set_provider(Box::new(provider));
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
        self.ecs.target_tps
    }

    pub fn tick(&mut self) -> Result<(), TickError> {
        // ================= PREPARE STAGE ================== //

        // ================= DISPATCH EVENTS STAGE ================= //

        // Prepare input manager
        self.input.prepare_dispatch(&mut self.resource);
        // Dispatch input events
        self.input.dispatch_events(&mut self.resource);
        // Dispatch platform events
        self.platform.dispatch_events();
        // Dispatch renderer events
        self.renderer.dispatch_events();

        // Dispatch network events

        // Dispatch storage events

        // TODO: dispatch more events ...

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        // Update ECS
        self.ecs
            .update(ECSUpdateContext {
                resource: &mut self.resource,
                input: &mut self.input,
                renderer: &mut self.renderer,
                platform: &mut self.platform,
                logger: &mut self.logger,
            })
            .map_err(|err| TickError::System)?;

        // ================= POST-UPDATE STAGE ================== //

        Ok(())
    }
}
