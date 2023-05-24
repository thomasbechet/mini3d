use mini3d_derive::Error;

use crate::asset::AssetManager;
use crate::disk::backend::DiskBackend;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::ecs::system::SystemCallback;
use crate::feature::{asset, component, system};
use crate::input::backend::{InputBackend, InputBackendError};
use crate::physics::PhysicsManager;
use crate::registry::RegistryManager;
use crate::registry::asset::Asset;
use crate::registry::component::Component;
use crate::registry::error::RegistryError;
use crate::renderer::RendererManager;
use crate::renderer::backend::{RendererBackend, RendererBackendError};
use crate::event::Events;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::script::ScriptManager;
use crate::serialize::{EncoderError, Serialize, DecoderError, Decoder};
use crate::uid::UID;
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
    pub(crate) script: ScriptManager,
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

        // Assets
        registry.assets.define_static::<asset::font::Font>(asset::font::Font::NAME)?;
        registry.assets.define_static::<asset::input_table::InputTable>(asset::input_table::InputTable::NAME)?;
        registry.assets.define_static::<asset::material::Material>(asset::material::Material::NAME)?;
        registry.assets.define_static::<asset::mesh::Mesh>(asset::mesh::Mesh::NAME)?;
        registry.assets.define_static::<asset::model::Model>(asset::model::Model::NAME)?;
        registry.assets.define_static::<asset::prefab::Prefab>(asset::prefab::Prefab::NAME)?;
        registry.assets.define_static::<asset::script::Script>(asset::script::Script::NAME)?;
        registry.assets.define_static::<asset::system_group::SystemGroup>(asset::system_group::SystemGroup::NAME)?;
        registry.assets.define_static::<asset::texture::Texture>(asset::texture::Texture::NAME)?;
        registry.assets.define_static::<asset::tilemap::Tilemap>(asset::tilemap::Tilemap::NAME)?;
        registry.assets.define_static::<asset::tileset::Tileset>(asset::tileset::Tileset::NAME)?;
        registry.assets.define_static::<asset::ui_template::UITemplate>(asset::ui_template::UITemplate::NAME)?;
        registry.assets.define_static::<asset::ui_stylesheet::UIStyleSheet>(asset::ui_stylesheet::UIStyleSheet::NAME)?;

        // Components
        registry.components.define_static::<component::camera::Camera>(component::camera::Camera::NAME)?;
        registry.components.define_static::<component::free_fly::FreeFly>(component::free_fly::FreeFly::NAME)?;
        registry.components.define_static::<component::lifecycle::Lifecycle>(component::lifecycle::Lifecycle::NAME)?;
        registry.components.define_static::<component::static_mesh::StaticMesh>(component::static_mesh::StaticMesh::NAME)?;
        registry.components.define_static::<component::rigid_body::RigidBody>(component::rigid_body::RigidBody::NAME)?;
        registry.components.define_static::<component::rotator::Rotator>(component::rotator::Rotator::NAME)?;
        registry.components.define_static::<component::transform::Transform>(component::transform::Transform::NAME)?;
        registry.components.define_static::<component::local_to_world::LocalToWorld>(component::local_to_world::LocalToWorld::NAME)?;
        registry.components.define_static::<component::hierarchy::Hierarchy>(component::hierarchy::Hierarchy::NAME)?;
        registry.components.define_static::<component::ui::UI>(component::ui::UI::NAME)?;
        registry.components.define_static::<component::ui::UIRenderTarget>(component::ui::UIRenderTarget::NAME)?;
        registry.components.define_static::<component::viewport::Viewport>(component::viewport::Viewport::NAME)?;
        registry.components.define_static::<component::canvas::Canvas>(component::canvas::Canvas::NAME)?;

        // Systems
        registry.systems.define_static("despawn_entities", system::despawn::run)?;
        registry.systems.define_static("renderer", system::renderer::despawn_renderer_entities)?;
        registry.systems.define_static("free_fly", system::free_fly::run)?;
        registry.systems.define_static("rotator", system::rotator::run)?;
        registry.systems.define_static("transform_propagate", system::transform::propagate)?;
        registry.systems.define_static("ui_update", system::ui::update)?;
        registry.systems.define_static("ui_render", system::ui::render)?;

        Ok(())
    }

    pub fn new(disk: impl DiskBackend + 'static) -> Self {
        let mut engine = Self {
            disk: Box::new(disk),
            registry: Default::default(),
            asset: Default::default(), 
            input: Default::default(), 
            script: Default::default(),
            ecs: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            accumulator: 0.0,
            time: 0.0,
            running: true,
        };
        engine.define_core_features().expect("Failed to define core features");
        engine
    }

    pub fn invoke_system(&mut self, system: UID) {
        self.ecs.invoke(system)
    }

    pub fn save_state(&self) -> Result<Box<[u8]>, EncoderError> {
        let mut buffer = Vec::new();
        self.asset.save_state(&mut buffer)?;
        self.renderer.save_state(&mut buffer)?;
        self.ecs.save_state(&mut buffer)?;
        self.input.save_state(&mut buffer)?;
        self.accumulator.serialize(&mut buffer)?;
        self.time.serialize(&mut buffer)?;
        self.running.serialize(&mut buffer)?;
        Ok(buffer.into_boxed_slice())
    }

    pub fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        self.asset.load_state(&self.registry.borrow().assets, decoder)?;
        self.renderer.load_state(decoder)?;
        self.ecs.load_state(&self.registry.borrow().components, decoder)?;
        self.input.load_state(decoder)?;
        self.accumulator = Serialize::deserialize(decoder, &Default::default())?;
        self.time = Serialize::deserialize(decoder, &Default::default())?;
        self.running = Serialize::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub fn define_static_component<C: Component>(&mut self, name: &str) -> Result<UID, RegistryError> {
        self.registry.borrow_mut().components.define_static::<C>(name)
    }

    pub fn define_static_system(&mut self, name: &str, system: SystemCallback) -> Result<UID, RegistryError> {
        self.registry.borrow_mut().systems.define_static(name, system)
    }

    pub fn define_static_asset<A: Asset>(&mut self, name: &str) -> Result<UID, RegistryError> {
        self.registry.borrow_mut().assets.define_static::<A>(name)
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
                SystemEvent::Shutdown => self.running = true,
            }
        }

        // TODO: dispatch more events ...

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        self.ecs.update(
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
            &mut self.script,
            fixed_update_count,
        ).map_err(|_| ProgressError::ECSError)?;

        Ok(())
    }

    pub fn synchronize_input(&mut self, backend: &mut impl InputBackend) -> Result<(), InputBackendError> {
        self.input.synchronize_backend(backend)
    }
    
    pub fn synchronize_renderer(&mut self, backend: &mut impl RendererBackend, reset: bool) -> Result<(), RendererBackendError> {
        if reset {
            backend.reset()?;
            self.renderer.reset(&mut self.ecs);
        }
        self.renderer.synchronize_backend(backend, &self.asset, &mut self.ecs)?;
        Ok(())
    }
}