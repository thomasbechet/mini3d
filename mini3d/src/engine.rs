use mini3d_derive::Error;

use crate::asset::AssetManager;
use crate::disk::backend::DiskBackend;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::event::system::SystemEvent;
use crate::event::Events;
use crate::feature::system::ui::RenderUI;
use crate::feature::{component, system};
use crate::input::backend::{InputBackend, InputBackendError};
use crate::input::InputManager;
use crate::physics::PhysicsManager;
use crate::registry::component::Component;
use crate::registry::error::RegistryError;
use crate::registry::RegistryManager;
use crate::renderer::backend::{RendererBackend, RendererBackendError};
use crate::renderer::RendererManager;
use crate::serialize::{Decoder, DecoderError, EncoderError, Serialize};
use crate::utils::uid::UID;
use core::cell::RefCell;

#[derive(Debug, Error)]
pub enum ProgressError {
    #[error("System error")]
    SystemError,
    #[error("ECS error")]
    ECSError,
}

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct Engine {
    pub(crate) disk: Box<dyn DiskBackend>,
    pub(crate) registry: RefCell<RegistryManager>,
    pub(crate) asset: AssetManager,
    pub(crate) input: InputManager,
    pub(crate) ecs: ECSManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    accumulator: f64,
    time: f64,
    running: bool,
}

impl Engine {
    fn define_core_features(&mut self) -> Result<(), RegistryError> {
        let mut registry = self.registry.borrow_mut();

        macro_rules! define_component {
            ($component: ty) => {
                registry
                    .components
                    .define_static::<$component>(<$component>::NAME)?;
            };
        }

        macro_rules! define_system {
            ($name: literal, $system: path) => {
                registry.systems.define_exclusive_callback($name, $system)?;
            };
        }

        // Define components
        define_component!(component::common::free_fly::FreeFly);
        define_component!(component::common::lifecycle::Lifecycle);
        define_component!(component::common::prefab::Prefab);
        define_component!(component::common::rotator::Rotator);
        define_component!(component::common::script::Script);
        define_component!(component::input::input_table::InputTable);
        define_component!(component::physics::rigid_body::RigidBody);
        define_component!(component::renderer::camera::Camera);
        define_component!(component::renderer::font::Font);
        define_component!(component::renderer::material::Material);
        define_component!(component::renderer::mesh::Mesh);
        define_component!(component::renderer::model::Model);
        define_component!(component::renderer::static_mesh::StaticMesh);
        define_component!(component::renderer::texture::Texture);
        define_component!(component::renderer::tilemap::Tilemap);
        define_component!(component::renderer::tileset::Tileset);
        define_component!(component::ui::canvas::Canvas);
        define_component!(component::ui::ui_stylesheet::UIStyleSheet);
        define_component!(component::ui::ui_template::UITemplate);
        define_component!(component::ui::ui::UI);
        define_component!(component::ui::ui::UIRenderTarget);
        define_component!(component::ui::viewport::Viewport);
        define_component!(component::scene::hierarchy::Hierarchy);
        define_component!(component::scene::local_to_world::LocalToWorld);
        define_component!(component::scene::transform::Transform);

        // Define systems
        define_system!("despawn_entities", system::despawn::run);
        define_system!("renderer", system::renderer::despawn_renderer_entities);
        define_system!("free_fly", system::free_fly::run);
        define_system!("rotator", system::rotator::run);
        define_system!("transform_propagate", system::transform::propagate);
        define_system!("ui_update", system::ui::update);
        define_system!("ui_render", system::ui::render);

        Ok(())
    }

    pub fn new(disk: impl DiskBackend + 'static) -> Self {
        let mut engine = Self {
            disk: Box::new(disk),
            registry: Default::default(),
            asset: Default::default(),
            input: Default::default(),
            ecs: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            accumulator: 0.0,
            time: 0.0,
            running: true,
        };
        engine
            .define_core_features()
            .expect("Failed to define core features");
        engine
    }

    pub fn invoke_system(&mut self, system: UID) {
        self.ecs.invoke(system)
    }

    pub fn save_state(&self) -> Result<Box<[u8]>, EncoderError> {
        let mut buffer = Vec::new();
        let registry = self.registry.borrow();
        self.asset.save_state(&registry.components, &mut buffer)?;
        self.renderer.save_state(&mut buffer)?;
        self.ecs.save_state(&registry.components, &mut buffer)?;
        self.input.save_state(&mut buffer)?;
        self.accumulator.serialize(&mut buffer)?;
        self.time.serialize(&mut buffer)?;
        self.running.serialize(&mut buffer)?;
        Ok(buffer.into_boxed_slice())
    }

    pub fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        self.asset
            .load_state(&self.registry.borrow().components, decoder)?;
        self.renderer.load_state(decoder)?;
        self.ecs
            .load_state(&self.registry.borrow().components, decoder)?;
        self.input.load_state(decoder)?;
        self.accumulator = Serialize::deserialize(decoder, &Default::default())?;
        self.time = Serialize::deserialize(decoder, &Default::default())?;
        self.running = Serialize::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub fn define_static_component<C: Component>(
        &mut self,
        name: &str,
    ) -> Result<(), RegistryError> {
        self.registry
            .borrow_mut()
            .components
            .define_static::<C>(name)
    }

    pub fn define_exclusive_callback(
        &mut self,
        name: &str,
        system: ExclusiveSystemCallback,
    ) -> Result<(), RegistryError> {
        self.registry
            .borrow_mut()
            .systems
            .define_exclusive_callback(name, system)?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn progress(&mut self, events: &Events, mut dt: f64) -> Result<(), ProgressError> {
        // ================= PREPARE STAGE ================== //

        // Reset graphics state
        self.renderer.prepare();

        // Compute delta time
        if dt > MAXIMUM_TIMESTEP {
            dt = MAXIMUM_TIMESTEP; // Slowing down
        }
        // Integrate time
        self.accumulator += dt;
        self.time += dt;
        // Compute number of fixed updates
        let fixed_update_count = (self.accumulator / FIXED_TIMESTEP) as u32;
        self.accumulator -= fixed_update_count as f64 * FIXED_TIMESTEP;

        // ================= DISPATCH STAGE ================= //

        // Prepare input manager
        self.input.prepare_dispatch();
        // Dispatch input events
        for event in &events.input {
            self.input.dispatch_event(event);
        }

        // Dispatch system events
        for event in &events.system {
            match event {
                SystemEvent::Shutdown => self.running = false,
            }
        }

        // TODO: dispatch more events ...

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        self.ecs
            .update(
                ECSUpdateContext {
                    registry: &self.registry,
                    asset: &mut self.asset,
                    input: &mut self.input,
                    renderer: &mut self.renderer,
                    events,
                    delta_time: dt,
                    time: self.time,
                    fixed_delta_time: FIXED_TIMESTEP,
                },
                fixed_update_count,
            )
            .map_err(|_| ProgressError::ECSError)?;

        Ok(())
    }

    pub fn synchronize_input(
        &mut self,
        backend: &mut impl InputBackend,
    ) -> Result<(), InputBackendError> {
        self.input.synchronize_backend(backend)
    }

    pub fn synchronize_renderer(
        &mut self,
        backend: &mut impl RendererBackend,
        reset: bool,
    ) -> Result<(), RendererBackendError> {
        if reset {
            backend.reset()?;
            self.renderer.reset(&mut self.ecs);
        }
        self.renderer
            .synchronize_backend(backend, &self.asset, &mut self.ecs)?;
        Ok(())
    }
}
