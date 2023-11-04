use mini3d_derive::Error;

use crate::activity::{ActivityEntry, ActivityId, ActivityManager};
use crate::disk::provider::DiskProvider;
use crate::disk::DiskManager;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::input::provider::InputProvider;
use crate::input::InputManager;
use crate::logger::provider::LoggerProvider;
use crate::logger::LoggerManager;
use crate::physics::PhysicsManager;
use crate::platform::provider::PlatformProvider;
use crate::platform::PlatformManager;
use crate::processor::Processor;
use crate::renderer::provider::RendererProvider;
use crate::renderer::RendererManager;
use crate::resource::error::ResourceError;
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
    input: bool,
    physics: bool,
    renderer: bool,
    ui: bool,
}

impl EngineFeatures {
    pub fn all() -> Self {
        Self {
            common: true,
            input: true,
            physics: true,
            renderer: true,
            ui: true,
        }
    }

    pub fn none() -> Self {
        Self {
            common: false,
            input: false,
            physics: false,
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

    fn define_types(&mut self, features: &EngineFeatures) {
        self.input.types.define(&mut self.resource);
        self.renderer.types.define(&mut self.resource);
    }

    fn register_core_features(&mut self, features: &EngineFeatures) -> Result<(), ResourceError> {
        // Define features

        if features.common {
            define_resource!(common::script::Script);
            define_resource!(common::program::Program);
            define_component!(common::free_fly::FreeFly, ComponentStorage::Single);
            define_component!(common::rotator::Rotator, ComponentStorage::Single);
            define_component!(common::transform::Transform, ComponentStorage::Single);
            define_component!(common::hierarchy::Hierarchy, ComponentStorage::Single);
            define_component!(
                common::local_to_world::LocalToWorld,
                ComponentStorage::Single
            );
            define_system_parallel!(common::free_fly::FreeFlySystem, SystemStage::UPDATE);
            define_system_parallel!(common::rotator::RotatorSystem, SystemStage::UPDATE);
            define_system_parallel!(common::transform::PropagateTransforms, SystemStage::UPDATE);
        }

        if features.input {
            define_resource!(input::action::InputAction);
            define_resource!(input::axis::InputAxis);
        }

        if features.physics {
            define_component!(physics::rigid_body::RigidBody, ComponentStorage::Single);
        }

        if features.renderer {
            define_resource!(renderer::font::Font);
            define_resource!(renderer::material::Material);
            define_resource!(renderer::mesh::Mesh);
            define_resource!(renderer::model::Model);
            define_resource!(renderer::texture::Texture);
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

        Ok(())
    }

    fn setup(&mut self, features: &EngineFeatures) {
        // Create root activity

        // Register core features
        self.register_core_features(features)
            .expect("Failed to define core features");
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
        engine.setup(&features);
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
