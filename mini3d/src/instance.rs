use mini3d_derive::Error;

use crate::asset::AssetManager;
use crate::disk::provider::DiskProvider;
use crate::disk::StorageManager;
use crate::ecs::scheduler::Invocation;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::feature::{common, input, physics, renderer};
use crate::input::provider::InputProvider;
use crate::input::InputManager;
use crate::logger::provider::LoggerProvider;
use crate::logger::LoggerManager;
use crate::physics::PhysicsManager;
use crate::platform::provider::PlatformProvider;
use crate::platform::PlatformManager;
use crate::registry::component::ComponentStorage;
use crate::registry::error::RegistryError;
use crate::registry::system::{ExclusiveSystem, SystemOrder, SystemStage};
use crate::registry::RegistryManager;
use crate::renderer::provider::RendererProvider;
use crate::renderer::RendererManager;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};
use crate::utils::uid::ToUID;

#[derive(Error, Debug)]
pub enum ProgressError {
    #[error("Core error")]
    Core,
    #[error("ECS system error")]
    System,
}

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;

#[derive(Clone)]
pub struct InstanceFeatures {
    common: bool,
    input: bool,
    physics: bool,
    renderer: bool,
    ui: bool,
}

impl InstanceFeatures {
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

impl Default for InstanceFeatures {
    fn default() -> Self {
        Self::all()
    }
}

pub struct Instance {
    pub(crate) registry: RegistryManager,
    pub(crate) storage: StorageManager,
    pub(crate) asset: AssetManager,
    pub(crate) input: InputManager,
    pub(crate) ecs: ECSManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    pub(crate) system: PlatformManager,
    pub(crate) logger: LoggerManager,
    global_time: f64,
}

impl Instance {
    fn register_core_features(&mut self, features: &InstanceFeatures) -> Result<(), RegistryError> {
        macro_rules! define_asset {
            ($asset: ty) => {
                self.registry.asset.add_static::<$asset>(<$asset>::NAME)?;
            };
        }

        macro_rules! define_component {
            ($component: ty, $storage: expr) => {
                self.registry
                    .component
                    .add_static::<$component>(<$component>::NAME, $storage)?;
            };
        }

        macro_rules! define_system_exclusive {
            ($system: ty, $stage: expr) => {
                self.registry.system.add_static_exclusive::<$system>(
                    <$system>::NAME,
                    $stage,
                    SystemOrder::default(),
                )?;
            };
        }

        macro_rules! define_system_parallel {
            ($system: ty, $stage: expr) => {
                self.registry.system.add_static_parallel::<$system>(
                    <$system>::NAME,
                    $stage,
                    SystemOrder::default(),
                )?;
            };
        }

        // Define features

        if features.common {
            define_asset!(common::script::Script);
            define_asset!(common::program::Program);
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
            define_asset!(input::action::InputAction);
            define_asset!(input::axis::InputAxis);
        }

        if features.physics {
            define_component!(physics::rigid_body::RigidBody, ComponentStorage::Single);
        }

        if features.renderer {
            define_asset!(renderer::font::Font);
            define_asset!(renderer::material::Material);
            define_asset!(renderer::mesh::Mesh);
            define_asset!(renderer::model::Model);
            define_asset!(renderer::texture::Texture);
            define_component!(renderer::camera::Camera, ComponentStorage::Single);
            define_component!(renderer::static_mesh::StaticMesh, ComponentStorage::Single);
            define_component!(renderer::tilemap::Tilemap, ComponentStorage::Single);
            define_component!(renderer::tileset::Tileset, ComponentStorage::Single);
            define_component!(renderer::viewport::Viewport, ComponentStorage::Single);
            define_component!(renderer::canvas::Canvas, ComponentStorage::Single);
            define_system_exclusive!(
                renderer::system::SynchronizeRendererResources,
                SystemStage::UPDATE
            );
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

    fn setup(&mut self, features: &InstanceFeatures) {
        // Register core features
        self.register_core_features(features)
            .expect("Failed to define core features");
        // Setup ECS
        self.ecs
            .scheduler
            .set_periodic_invoke(SystemStage::FIXED_UPDATE_60HZ, 1.0 / 60.0);
        // Update ECS and assets
        self.ecs.scheduler.on_registry_update(&self.registry.system);
        self.asset.on_registry_update(&self.registry.asset);
        // Setup managers
        self.renderer
            .reload_components_and_assets(&self.registry)
            .expect("Failed to reload component handles");
    }

    pub fn new(features: InstanceFeatures) -> Self {
        let mut instance = Self {
            registry: Default::default(),
            storage: Default::default(),
            asset: Default::default(),
            input: Default::default(),
            ecs: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            system: Default::default(),
            logger: Default::default(),
            global_time: 0.0,
        };
        instance.setup(&features);
        instance
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

    pub fn register_system<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: &str,
    ) -> Result<(), RegistryError> {
        self.registry
            .system
            .add_static_exclusive::<S>(name, stage, SystemOrder::default())?;
        self.ecs.on_registry_update(&self.registry)?;
        Ok(())
    }

    pub fn invoke(
        &mut self,
        stage: impl ToUID,
        invocation: Invocation,
    ) -> Result<(), RegistryError> {
        self.ecs.scheduler.invoke(stage.to_uid(), invocation)
    }

    pub fn save(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.asset.save_state(&self.registry.component, encoder)?;
        self.renderer.save_state(encoder)?;
        self.ecs.save_state(&self.registry.component, encoder)?;
        self.input.save_state(encoder)?;
        self.global_time.serialize(encoder)?;
        Ok(())
    }

    pub fn load(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        self.asset.load_state(&self.registry.component, decoder)?;
        self.renderer.load_state(decoder)?;
        self.ecs.load_state(&self.registry.component, decoder)?;
        self.input.load_state(decoder)?;
        self.global_time = Serialize::deserialize(decoder, &Default::default())?;
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
        self.input.prepare_dispatch();
        // Dispatch input events
        self.input.dispatch_events();
        // Dispatch system events
        self.system.dispatch_events();
        // Dispatch renderer events
        self.renderer.dispatch_events();

        // Dispatch network events

        // Dispatch storage events

        // TODO: dispatch more events ...

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        self.ecs
            .update(ECSUpdateContext {
                registry: &mut self.registry,
                asset: &mut self.asset,
                input: &mut self.input,
                renderer: &mut self.renderer,
                system: &mut self.system,
                logger: &mut self.logger,
                delta_time,
                global_time: self.global_time,
            })
            .map_err(|err| ProgressError::System)?;

        // ================= POST-UPDATE STAGE ================== //
        self.renderer
            .submit_graphics(&mut self.asset, &self.ecs.containers);

        Ok(())
    }
}
