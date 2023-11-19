use mini3d_derive::Error;

use crate::activity::{ActivityEntry, ActivityManager};
use crate::api::time::TimeAPI;
use crate::api::Context;
use crate::disk::provider::DiskProvider;
use crate::disk::DiskManager;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::feature::core::resource::ResourceType;
use crate::feature::ecs::component::{ComponentStorage, ComponentType};
use crate::feature::ecs::system::{System, SystemStage};
use crate::feature::{common, core, ecs, input, renderer};
use crate::input::provider::InputProvider;
use crate::input::InputManager;
use crate::logger::provider::LoggerProvider;
use crate::logger::LoggerManager;
use crate::physics::PhysicsManager;
use crate::platform::provider::PlatformProvider;
use crate::platform::PlatformManager;
use crate::renderer::provider::RendererProvider;
use crate::renderer::RendererManager;
use crate::resource::ResourceManager;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};

#[derive(Error, Debug)]
pub enum ProgressError {
    #[error("Core error")]
    Core,
    #[error("ECS system error")]
    System,
}

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;

#[derive(Clone)]
pub struct EngineConfig {
    bootstrap: Option<fn(&mut Context)>,
    common: bool,
    renderer: bool,
    ui: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            bootstrap: None,
            common: true,
            renderer: true,
            ui: true,
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
    pub(crate) activity: ActivityManager,
    pub(crate) ecs: ECSManager,
    pub(crate) resource: ResourceManager,
    pub(crate) storage: DiskManager,
    pub(crate) input: InputManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    pub(crate) platform: PlatformManager,
    pub(crate) logger: LoggerManager,
    global_time: f64,
}

impl Engine {
    fn setup_root_activity(&mut self) {
        self.activity.root = self.activity.activities.add(ActivityEntry {
            name: "root".into(),
            parent: Default::default(),
            ecs: Default::default(),
        });
        self.activity.activities[self.activity.root].ecs = self.ecs.add(self.activity.root);
        self.activity.active = self.activity.root;
    }

    fn setup_resource_manager(&mut self) {
        self.resource.define_meta_type(self.activity.root);
    }

    fn define_resource_types(&mut self, config: &EngineConfig) {
        macro_rules! define_resource {
            ($resource: ty) => {
                self.resource
                    .create_resource_type(
                        Some(<$resource>::NAME),
                        self.activity.root,
                        ResourceType::native::<$resource>(),
                    )
                    .unwrap()
            };
        }

        self.ecs.handles.component = define_resource!(ecs::component::ComponentType);
        self.ecs.handles.system = define_resource!(ecs::system::System);
        self.ecs.handles.system_set = define_resource!(ecs::system::SystemSet);
        self.ecs.handles.system_stage = define_resource!(ecs::system::SystemStage);

        self.input.handles.action = define_resource!(input::action::InputAction);
        self.input.handles.axis = define_resource!(input::axis::InputAxis);
        self.input.handles.text = define_resource!(input::text::InputText);

        self.renderer.handles.font = define_resource!(renderer::font::Font);
        self.renderer.handles.material = define_resource!(renderer::material::Material);
        self.renderer.handles.mesh = define_resource!(renderer::mesh::Mesh);
        self.renderer.handles.texture = define_resource!(renderer::texture::Texture);
        self.renderer.handles.graph = define_resource!(renderer::graph::RenderGraph);
        self.renderer.handles.model = define_resource!(renderer::model::Model);
        self.renderer.handles.array = define_resource!(renderer::array::RenderArray);
        self.renderer.handles.constant = define_resource!(renderer::constant::RenderConstant);
        self.renderer.handles.command_buffer =
            define_resource!(renderer::command::RenderCommandBuffer);
        self.renderer.handles.graphics_pipeline =
            define_resource!(renderer::pipeline::GraphicsPipeline);
        self.renderer.handles.compute_pipeline =
            define_resource!(renderer::pipeline::ComputePipeline);

        define_resource!(core::activity::Activity);
        define_resource!(core::structure::StructDefinition);

        if config.common {
            define_resource!(common::script::Script);
            define_resource!(common::program::Program);
        }
    }

    fn define_component_types(&mut self, config: &EngineConfig) {
        macro_rules! define_component {
            ($component: ty, $storage: expr) => {
                self.resource
                    .create(
                        Some(<$component>::NAME),
                        self.ecs.handles.component,
                        self.activity.root,
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
                        self.activity.root,
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
                        self.activity.root,
                        System::native_parallel::<$system>(),
                    )
                    .unwrap()
            };
        }

        if config.common {
            define_component!(common::free_fly::FreeFly, ComponentStorage::Single);
            define_component!(common::rotator::Rotator, ComponentStorage::Single);
            define_component!(common::transform::Transform, ComponentStorage::Single);
            define_component!(common::hierarchy::Hierarchy, ComponentStorage::Single);
            define_component!(
                common::local_to_world::LocalToWorld,
                ComponentStorage::Single
            );

            define_parallel_system!(common::free_fly::FreeFlySystem);
            define_parallel_system!(common::rotator::RotatorSystem);
            define_parallel_system!(common::transform::PropagateTransforms);
        }

        if config.renderer {
            define_component!(renderer::camera::Camera, ComponentStorage::Single);
            define_component!(renderer::static_mesh::StaticMesh, ComponentStorage::Single);
            define_component!(renderer::tilemap::Tilemap, ComponentStorage::Single);
            define_component!(renderer::tileset::Tileset, ComponentStorage::Single);
            define_component!(renderer::viewport::Viewport, ComponentStorage::Single);
            define_component!(renderer::canvas::Canvas, ComponentStorage::Single);
        }

        if config.ui {
            // define_component!(ui::ui_stylesheet::UIStyleSheet);
            // define_component!(ui::ui::UI);
            // define_component!(ui::ui::UIRenderTarget);
            // define_system_parallel!(ui::update_ui::UpdateUI, SystemStage::UPDATE);
            // define_system_exclusive!(ui::render_ui::RenderUI, SystemStage::UPDATE);
        }

        self.ecs.handles.update_stage = self
            .resource
            .create(
                Some(SystemStage::UPDATE),
                self.ecs.handles.system_stage,
                self.activity.root,
                SystemStage::default(),
            )
            .unwrap()
            .into();
        self.ecs.handles.start_stage = self
            .resource
            .create(
                Some(SystemStage::START),
                self.ecs.handles.system_stage,
                self.activity.root,
                SystemStage::default(),
            )
            .unwrap()
            .into();
        self.resource
            .create(
                Some(SystemStage::UPDATE_60HZ),
                self.ecs.handles.system_stage,
                self.activity.root,
                SystemStage::periodic(1.0 / 60.0),
            )
            .unwrap();
    }

    fn run_bootstrap(&mut self, config: &EngineConfig) {
        if let Some(bootstrap) = config.bootstrap {
            let (root, _) = &mut self.ecs.instances[self.activity.root];
            bootstrap(&mut Context {
                activity: &mut self.activity,
                resource: &mut self.resource,
                input: &mut self.input,
                renderer: &mut self.renderer,
                platform: &mut self.platform,
                logger: &mut self.logger,
                time: TimeAPI {
                    delta: 0.0,
                    global: 0.0,
                },
                ecs: root,
                ecs_types: &self.ecs.handles,
            });
            // Flush activity commands
            self.activity
                .flush_commands(&mut self.ecs, &mut self.resource);
        }
    }

    pub fn new(config: EngineConfig) -> Self {
        let mut engine = Self {
            activity: Default::default(),
            ecs: Default::default(),
            storage: Default::default(),
            resource: Default::default(),
            input: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            platform: Default::default(),
            logger: Default::default(),
            global_time: 0.0,
        };
        engine.setup_root_activity();
        engine.setup_resource_manager();
        engine.define_resource_types(&config);
        engine.define_component_types(&config);
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

    pub fn progress(&mut self, mut delta_time: f64) -> Result<(), ProgressError> {
        // ================= PREPARE STAGE ================== //

        // Compute delta time
        if delta_time > MAXIMUM_TIMESTEP {
            delta_time = MAXIMUM_TIMESTEP; // Slowing down
        }
        // Integrate time
        self.global_time += delta_time;

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
                activity: &mut self.activity,
                resource: &mut self.resource,
                input: &mut self.input,
                renderer: &mut self.renderer,
                platform: &mut self.platform,
                logger: &mut self.logger,
                delta_time,
                global_time: self.global_time,
            })
            .map_err(|err| ProgressError::System)?;

        // Flush activity commands
        self.activity
            .flush_commands(&mut self.ecs, &mut self.resource);

        // ================= POST-UPDATE STAGE ================== //

        Ok(())
    }
}
