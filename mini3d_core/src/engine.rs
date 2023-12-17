use alloc::boxed::Box;
use mini3d_derive::{fixed, Error};

use crate::disk::provider::DiskProvider;
use crate::disk::DiskManager;
use crate::ecs::api::time::TimeAPI;
use crate::ecs::api::Context;
use crate::ecs::resource::{ComponentStorage, ComponentType, System, SystemStage};
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
use crate::resource::ResourceManager;
use crate::resource::ResourceType;
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
pub struct EngineConfig {
    bootstrap: Option<fn(&mut Context)>,
    renderer: bool,
    target_tps: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            bootstrap: None,
            renderer: true,
            target_tps: 60,
        }
    }
}

impl EngineConfig {
    pub fn bootstrap(mut self, bootstrap: fn(&mut Context)) -> Self {
        self.bootstrap = Some(bootstrap);
        self
    }
}

pub struct Engine {
    pub(crate) ecs: ECSManager,
    pub(crate) resource: ResourceManager,
    pub(crate) storage: DiskManager,
    pub(crate) input: InputManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    pub(crate) platform: PlatformManager,
    pub(crate) logger: LoggerManager,
}

impl Engine {
    fn setup_resource_manager(&mut self) {
        self.resource.define_meta_type();
    }

    fn define_resource_types(&mut self, config: &EngineConfig) {
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

        self.ecs.handles.component = define_resource!(ecs::resource::ComponentType);
        self.ecs.handles.system = define_resource!(ecs::resource::System);
        self.ecs.handles.system_set = define_resource!(ecs::resource::SystemSet);
        self.ecs.handles.system_stage = define_resource!(ecs::resource::SystemStage);

        self.input.handles.action = define_resource!(input::resource::InputAction);
        self.input.handles.axis = define_resource!(input::resource::InputAxis);
        self.input.handles.text = define_resource!(input::resource::InputText);

        self.renderer.handles.font = define_resource!(renderer::resource::Font);
        self.renderer.handles.material = define_resource!(renderer::resource::Material);
        self.renderer.handles.mesh = define_resource!(renderer::resource::Mesh);
        self.renderer.handles.texture = define_resource!(renderer::resource::Texture);
        self.renderer.handles.model = define_resource!(renderer::resource::Model);
        self.renderer.handles.transform = define_resource!(renderer::resource::RenderTransform);

        define_resource!(script::resource::structure::StructDefinition);
        define_resource!(script::resource::script::Script);
        define_resource!(script::resource::program::Program);
    }

    fn define_component_types(&mut self, config: &EngineConfig) {
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

        self.ecs.handles.start_stage = self
            .resource
            .create(
                Some(SystemStage::START),
                self.ecs.handles.system_stage,
                SystemStage::default(),
            )
            .unwrap()
            .into();
        self.ecs.handles.tick_stage = self
            .resource
            .create(
                Some(SystemStage::TICK),
                self.ecs.handles.system_stage,
                SystemStage::default(),
            )
            .unwrap()
            .into();
    }

    fn setup_ecs(&mut self, config: &EngineConfig) {
        self.ecs.target_tps = config.target_tps;
        self.ecs
            .scheduler
            .invoke(self.ecs.handles.start_stage, Invocation::Immediate);
    }

    fn run_bootstrap(&mut self, config: &EngineConfig) {
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
                ecs_types: &self.ecs.handles,
                commands: &mut self.ecs.commands,
            });
            self.ecs.flush_commands(&mut self.resource);
        }
    }

    pub fn new(config: EngineConfig) -> Self {
        let mut engine = Self {
            ecs: Default::default(),
            storage: Default::default(),
            resource: Default::default(),
            input: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            platform: Default::default(),
            logger: Default::default(),
        };
        engine.setup_resource_manager();
        engine.define_resource_types(&config);
        engine.define_component_types(&config);
        engine.setup_ecs(&config);
        engine.run_bootstrap(&config);
        engine
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
