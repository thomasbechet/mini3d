use mini3d_derive::Error;

use crate::activity::{ActivityEntry, ActivityId, ActivityManager};
use crate::disk::provider::DiskProvider;
use crate::disk::DiskManager;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::feature::core::resource::ResourceType;
use crate::feature::ecs::component::{ComponentStorage, ComponentType};
use crate::feature::ecs::system::System;
use crate::feature::{common, ecs, input, renderer};
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
pub struct EngineFeatures {
    common: bool,
    renderer: bool,
    ui: bool,
}

impl EngineFeatures {
    pub fn all() -> Self {
        Self {
            common: true,
            renderer: true,
            ui: true,
        }
    }

    pub fn none() -> Self {
        Self {
            common: false,
            renderer: false,
            ui: false,
        }
    }
}

impl Default for EngineFeatures {
    fn default() -> Self {
        Self::all()
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
    pub(crate) system: PlatformManager,
    pub(crate) logger: LoggerManager,
    global_time: f64,
}

impl Engine {
    fn setup_root_activity(&mut self) {
        self.activity.root = ActivityId(self.activity.entries.add(ActivityEntry {
            name: "root".into(),
            parent: Default::default(),
            ecs: Default::default(),
        }));
        self.activity.entries[self.activity.root.0].ecs = self.ecs.add(self.activity.root);
        self.activity.active = self.activity.root;
    }

    fn setup_resource_manager(&mut self) {
        self.resource.define_meta_type(self.activity.root);
    }

    fn define_resource_types(&mut self, features: &EngineFeatures) {
        macro_rules! define_resource {
            ($resource: ty) => {
                self.resource
                    .add_resource_type(
                        ResourceType::native::<$resource>(),
                        self.activity.root,
                        Some(<$resource>::NAME),
                    )
                    .unwrap()
            };
        }

        self.ecs.types.component = define_resource!(ecs::component::ComponentType);
        self.ecs.types.system = define_resource!(ecs::system::System);
        self.ecs.types.system_set = define_resource!(ecs::system::SystemSet);
        self.ecs.types.system_stage = define_resource!(ecs::system::SystemStage);

        self.input.types.action = define_resource!(input::action::InputAction);
        self.input.types.axis = define_resource!(input::axis::InputAxis);
        self.input.types.text = define_resource!(input::text::InputText);

        self.renderer.types.font = define_resource!(renderer::font::Font);
        self.renderer.types.material = define_resource!(renderer::material::Material);
        self.renderer.types.mesh = define_resource!(renderer::mesh::Mesh);
        self.renderer.types.texture = define_resource!(renderer::texture::Texture);

        if features.common {
            define_resource!(common::script::Script);
            define_resource!(common::program::Program);
        }
    }

    fn define_component_types(&mut self, features: &EngineFeatures) {
        macro_rules! define_component {
            ($component: ty, $storage: expr) => {
                self.resource
                    .add(
                        ComponentType::native::<$component>($storage),
                        self.ecs.types.component,
                        self.activity.root,
                        Some(<$component>::NAME),
                    )
                    .unwrap()
            };
        }

        macro_rules! define_exclusive_system {
            ($system: ty) => {
                self.resource
                    .add(
                        System::native_exclusive::<$system>(),
                        self.ecs.types.system,
                        self.activity.root,
                        Some(<$system>::NAME),
                    )
                    .unwrap()
            };
        }

        macro_rules! define_parallel_system {
            ($system: ty) => {
                self.resource
                    .add(
                        System::native_parallel::<$system>(),
                        self.ecs.types.system,
                        self.activity.root,
                        Some(<$system>::NAME),
                    )
                    .unwrap()
            };
        }

        if features.common {
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

        if features.renderer {
            define_component!(renderer::camera::Camera, ComponentStorage::Single);
            define_component!(renderer::static_mesh::StaticMesh, ComponentStorage::Single);
            define_component!(renderer::tilemap::Tilemap, ComponentStorage::Single);
            define_component!(renderer::tileset::Tileset, ComponentStorage::Single);
            define_component!(renderer::viewport::Viewport, ComponentStorage::Single);
            define_component!(renderer::canvas::Canvas, ComponentStorage::Single);
        }

        if features.ui {
            // define_component!(ui::ui_stylesheet::UIStyleSheet);
            // define_component!(ui::ui::UI);
            // define_component!(ui::ui::UIRenderTarget);
            // define_system_parallel!(ui::update_ui::UpdateUI, SystemStage::UPDATE);
            // define_system_exclusive!(ui::render_ui::RenderUI, SystemStage::UPDATE);
        }
    }

    pub fn new(features: EngineFeatures) -> Self {
        let mut engine = Self {
            activity: Default::default(),
            ecs: Default::default(),
            storage: Default::default(),
            resource: Default::default(),
            input: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            system: Default::default(),
            logger: Default::default(),
            global_time: 0.0,
        };
        engine.setup_root_activity();
        engine.setup_resource_manager();
        engine.define_resource_types(&features);
        engine
    }

    pub fn set_renderer_provider(&mut self, provider: impl RendererProvider + 'static) {
        self.renderer.set_provider(Box::new(provider));
    }

    pub fn set_input_provider(&mut self, provider: impl InputProvider + 'static) {
        self.input.set_provider(Box::new(provider));
    }

    pub fn set_system_provider(&mut self, provider: impl PlatformProvider + 'static) {
        self.system.set_provider(Box::new(provider));
    }

    pub fn set_storage_provider(&mut self, provider: impl DiskProvider + 'static) {
        self.storage.set_provider(Box::new(provider));
    }

    pub fn set_logger_provider(&mut self, provider: impl LoggerProvider + 'static) {
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

        // Reset graphics state
        self.renderer.prepare();

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
        // Dispatch system events
        self.system.dispatch_events();
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
                system: &mut self.system,
                logger: &mut self.logger,
                delta_time,
                global_time: self.global_time,
            })
            .map_err(|err| ProgressError::System)?;

        // Flush activity commands
        self.activity
            .flush_commands(&mut self.ecs, &mut self.resource);

        // ================= POST-UPDATE STAGE ================== //
        self.renderer
            .submit_graphics(&mut self.resource, &self.ecs.containers);

        Ok(())
    }
}
