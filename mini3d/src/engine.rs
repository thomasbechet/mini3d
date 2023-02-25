use anyhow::{Result, Context};
use serde::de::{Visitor, DeserializeSeed};
use serde::ser::SerializeTuple;
use serde::{Serializer, Deserializer, Serialize};

use crate::asset::{AssetManager, AssetEntry};
use crate::ecs::ECSManager;
use crate::feature::{asset, component, system};
use crate::physics::PhysicsManager;
use crate::registry::RegistryManager;
use crate::registry::asset::Asset;
use crate::registry::component::Component;
use crate::registry::system::SystemCallback;
use crate::renderer::RendererManager;
use crate::renderer::backend::RendererBackend;
use crate::event::Events;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::request::Requests;
use crate::script::ScriptManager;
use crate::uid::UID;
use core::cell::RefCell;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct Engine {
    pub(crate) registry: RefCell<RegistryManager>,
    pub(crate) asset: RefCell<AssetManager>,
    pub(crate) input: RefCell<InputManager>,
    pub(crate) script: RefCell<ScriptManager>,
    pub(crate) ecs: RefCell<ECSManager>,
    pub(crate) renderer: RefCell<RendererManager>,
    pub(crate) physics: RefCell<PhysicsManager>,
    accumulator: f64,
    time: f64,
}

impl Engine {

    fn define_core_features(&mut self) -> Result<()> {

        // Assets
        self.define_asset::<asset::font::Font>(asset::font::Font::NAME)?;
        self.define_asset::<asset::input_action::InputAction>(asset::input_action::InputAction::NAME)?;
        self.define_asset::<asset::input_axis::InputAxis>(asset::input_axis::InputAxis::NAME)?;
        self.define_asset::<asset::input_table::InputTable>(asset::input_table::InputTable::NAME)?;
        self.define_asset::<asset::material::Material>(asset::material::Material::NAME)?;
        self.define_asset::<asset::mesh::Mesh>(asset::mesh::Mesh::NAME)?;
        self.define_asset::<asset::model::Model>(asset::model::Model::NAME)?;
        self.define_asset::<asset::rhai_script::RhaiScript>(asset::rhai_script::RhaiScript::NAME)?;
        self.define_asset::<asset::system_group::SystemGroup>(asset::system_group::SystemGroup::NAME)?;
        self.define_asset::<asset::texture::Texture>(asset::texture::Texture::NAME)?;
        self.define_asset::<asset::tilemap::Tilemap>(asset::tilemap::Tilemap::NAME)?;
        self.define_asset::<asset::tileset::Tileset>(asset::tileset::Tileset::NAME)?;
        self.define_asset::<asset::ui_template::UITemplate>(asset::ui_template::UITemplate::NAME)?;
        self.define_asset::<asset::world_template::WorldTemplate>(asset::world_template::WorldTemplate::NAME)?;

        // Components
        self.define_component::<component::camera::Camera>(component::camera::Camera::NAME)?;
        self.define_component::<component::free_fly::FreeFly>(component::free_fly::FreeFly::NAME)?;
        self.define_component::<component::lifecycle::Lifecycle>(component::lifecycle::Lifecycle::NAME)?;
        self.define_component::<component::model::Model>(component::model::Model::NAME)?;
        self.define_component::<component::rhai_scripts::RhaiScripts>(component::rhai_scripts::RhaiScripts::NAME)?;
        self.define_component::<component::rigid_body::RigidBody>(component::rigid_body::RigidBody::NAME)?;
        self.define_component::<component::rotator::Rotator>(component::rotator::Rotator::NAME)?;
        self.define_component::<component::script_storage::ScriptStorage>(component::script_storage::ScriptStorage::NAME)?;
        self.define_component::<component::transform::Transform>(component::transform::Transform::NAME)?;
        self.define_component::<component::local_to_world::LocalToWorld>(component::local_to_world::LocalToWorld::NAME)?;
        self.define_component::<component::hierarchy::Hierarchy>(component::hierarchy::Hierarchy::NAME)?;
        self.define_component::<component::ui::UIComponent>(component::ui::UIComponent::NAME)?;
        self.define_component::<component::viewport::Viewport>(component::viewport::Viewport::NAME)?;
        self.define_component::<component::canvas::Canvas>(component::canvas::Canvas::NAME)?;

        // Systems
        self.define_system("despawn_entities", system::despawn::run)?;
        self.define_system("renderer", system::renderer::despawn_renderer_entities)?;
        self.define_system("free_fly", system::free_fly::run)?;
        self.define_system("rhai_update_scripts", system::rhai::update_scripts)?;
        self.define_system("rotator", system::rotator::run)?;
        self.define_system("transform_propagate", system::transform::propagate)?;
        self.define_system("ui_update", system::ui::update)?;
        self.define_system("ui_render", system::ui::render)?;

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let mut engine = Self {
            registry: Default::default(),
            asset: Default::default(), 
            input: Default::default(), 
            script: Default::default(),
            ecs: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            accumulator: 0.0,
            time: 0.0,
        };
        engine.define_core_features()?;
        Ok(engine)
    }

    pub fn define_asset<A: Asset>(&mut self, name: &str) -> Result<()> {
        self.registry.borrow_mut().assets.define_compiled::<A>(name)
    }

    pub fn define_component<C: Component>(&mut self, name: &str) -> Result<()> {
        self.registry.borrow_mut().components.define_compiled::<C>(name)
    }

    pub fn define_system(&mut self, name: &str, system: SystemCallback) -> Result<()> {
        self.registry.borrow_mut().systems.define_compiled(name, system)
    }

    pub fn iter_asset<A: Asset>(&'_ mut self, asset: UID) -> Result<impl Iterator<Item = (&UID, &'_ AssetEntry<A>)>> {
        self.asset.get_mut().iter::<A>(asset)
    }

    pub fn asset_entry<A: Asset>(&'_ mut self, asset: UID, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.asset.get_mut().entry::<A>(asset, uid)
    }

    pub fn save_state<S: Serializer>(&mut self, serializer: S) -> Result<S::Ok, S::Error> {
        struct AssetManagerSerialize<'a> {
            manager: &'a AssetManager,
        }
        impl<'a> Serialize for AssetManagerSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.save_state(serializer)
            }
        }
        struct RendererManagerSerialize<'a> {
            manager: &'a RendererManager,
        }
        impl<'a> Serialize for RendererManagerSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.save_state(serializer)
            }
        }
        struct ECSManagerSerialize<'a> {
            manager: &'a ECSManager,
            registry: &'a RegistryManager,
        }
        impl<'a> Serialize for ECSManagerSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.save_state(self.registry, serializer)
            }
        }
        struct InputManagerSerialize<'a> {
            manager: &'a InputManager,
        }
        impl<'a> Serialize for InputManagerSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.save_state(serializer)
            }
        }
        let mut tuple = serializer.serialize_tuple(6)?;
        tuple.serialize_element(&AssetManagerSerialize { manager: self.asset.get_mut() })?;
        tuple.serialize_element(&RendererManagerSerialize { manager: self.renderer.get_mut() })?;
        tuple.serialize_element(&ECSManagerSerialize { manager: self.ecs.get_mut(), registry: &self.registry.borrow() })?;
        tuple.serialize_element(&InputManagerSerialize { manager: self.input.get_mut() })?;
        tuple.serialize_element(&self.accumulator)?;
        tuple.serialize_element(&self.time)?;
        tuple.end()
    }

    pub fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct EngineVisitor<'a> {
            engine: &'a mut Engine,
        }
        impl<'de, 'a> Visitor<'de> for EngineVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("App")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: serde::de::SeqAccess<'de> {
                use serde::de::Error;
                struct AssetManagerDeserializeSeed<'a> {
                    manager: &'a mut AssetManager,
                    registry: &'a RegistryManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for AssetManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(&self.registry.assets, deserializer)
                    }
                }
                struct RendererManagerDeserializeSeed<'a> {
                    manager: &'a mut RendererManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for RendererManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(deserializer)
                    }
                }
                struct ECSManagerDeserializeSeed<'a> {
                    manager: &'a mut ECSManager,
                    registry: &'a RegistryManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ECSManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(self.registry, deserializer)
                    }
                }
                struct InputManagerDeserializeSeed<'a> {
                    manager: &'a mut InputManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for InputManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(deserializer)
                    }
                }
                seq.next_element_seed(AssetManagerDeserializeSeed { manager: self.engine.asset.get_mut(), registry: &self.engine.registry.borrow() })?;
                seq.next_element_seed(RendererManagerDeserializeSeed { manager: self.engine.renderer.get_mut() })?;
                seq.next_element_seed(ECSManagerDeserializeSeed { manager: self.engine.ecs.get_mut(), registry: &self.engine.registry.borrow() })?;
                seq.next_element_seed(InputManagerDeserializeSeed { manager: self.engine.input.get_mut() })?;
                self.engine.accumulator = seq.next_element()?.with_context(|| "Expect accumulator").map_err(Error::custom)?;
                self.engine.time = seq.next_element()?.with_context(|| "Expect time").map_err(Error::custom)?;
                self.engine.renderer.get_mut().reset(self.engine.ecs.get_mut()).map_err(Error::custom)?;
                Ok(())
            }
        }
        deserializer.deserialize_tuple(7, EngineVisitor { engine: self })?;
        Ok(())
    }

    pub fn progress(
        &mut self,
        events: &Events,
        requests: &mut Requests,
        mut delta_time: f64,
    ) -> Result<()> {

        // ================= PREPARE STAGE ================== //

        // Reset graphics state
        self.renderer.get_mut().prepare()?;

        // Compute delta time
        if delta_time > MAXIMUM_TIMESTEP {
            delta_time = MAXIMUM_TIMESTEP; // Slowing down
        }
        // Integrate time
        self.accumulator += delta_time;
        self.time += delta_time;
        // Compute number of fixed updates
        let fixed_update_count = (self.accumulator / FIXED_TIMESTEP) as u32;
        self.accumulator -= fixed_update_count as f64 * FIXED_TIMESTEP;

        // ================= DISPATCH STAGE ================= //

        // Prepare input manager
        self.input.get_mut().prepare_dispatch();
        // Dispatch input events
        for event in &events.input {
            self.input.get_mut().dispatch_event(event);
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

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        self.ecs.get_mut().update(&self.registry, &self.asset, &self.input, &self.renderer, &self.script, delta_time, self.time, FIXED_TIMESTEP, fixed_update_count)?;

        // ================= REQUESTS STAGE ================= //

        // Check input requests
        if self.input.get_mut().reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input.get_mut().reload_input_mapping = false;
        }

        Ok(())
    }

    pub fn update_renderer(
        &mut self,
        backend: &mut impl RendererBackend,
        reset: bool,
    ) -> Result<()> {
        if reset {
            backend.reset()?;
            self.renderer.get_mut().reset(self.ecs.get_mut())?;
        }
        self.renderer.get_mut().update_backend(backend, self.asset.get_mut(), self.ecs.get_mut())?;
        Ok(())
    }
}