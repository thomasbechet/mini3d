use anyhow::{Result, Context};
use serde::de::{Visitor, DeserializeSeed};
use serde::ser::SerializeTuple;
use serde::{Serializer, Deserializer, Serialize};

use crate::asset::AssetManager;
use crate::ecs::ECSManager;
use crate::ecs::system::SystemCallback;
use crate::feature::asset::input_table::{InputTable, InputAction, InputAxis};
use crate::feature::{asset, component, system};
use crate::physics::PhysicsManager;
use crate::registry::RegistryManager;
use crate::renderer::RendererManager;
use crate::renderer::backend::RendererBackend;
use crate::event::Events;
use crate::event::system::SystemEvent;
use crate::input::{InputManager, InputActionState, InputAxisState};
use crate::request::Requests;
use crate::script::ScriptManager;
use core::cell::RefCell;
use std::cell::Ref;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct Engine {
    pub(crate) registry: RefCell<RegistryManager>,
    pub(crate) asset: AssetManager,
    pub(crate) input: InputManager,
    pub(crate) script: ScriptManager,
    pub(crate) ecs: ECSManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    accumulator: f64,
    time: f64,
}

impl Engine {

    fn define_core_features(&mut self) -> Result<()> {

        let mut registry = self.registry.borrow_mut();

        // Assets
        registry.assets.define_static::<asset::font::Font>(asset::font::Font::NAME)?;
        registry.assets.define_static::<asset::input_table::InputTable>(asset::input_table::InputTable::NAME)?;
        registry.assets.define_static::<asset::material::Material>(asset::material::Material::NAME)?;
        registry.assets.define_static::<asset::mesh::Mesh>(asset::mesh::Mesh::NAME)?;
        registry.assets.define_static::<asset::model::Model>(asset::model::Model::NAME)?;
        registry.assets.define_static::<asset::rhai_script::RhaiScript>(asset::rhai_script::RhaiScript::NAME)?;
        registry.assets.define_static::<asset::system_group::SystemGroup>(asset::system_group::SystemGroup::NAME)?;
        registry.assets.define_static::<asset::texture::Texture>(asset::texture::Texture::NAME)?;
        registry.assets.define_static::<asset::tilemap::Tilemap>(asset::tilemap::Tilemap::NAME)?;
        registry.assets.define_static::<asset::tileset::Tileset>(asset::tileset::Tileset::NAME)?;
        registry.assets.define_static::<asset::ui_template::UITemplate>(asset::ui_template::UITemplate::NAME)?;
        registry.assets.define_static::<asset::ui_stylesheet::UIStyleSheet>(asset::ui_stylesheet::UIStyleSheet::NAME)?;
        registry.assets.define_static::<asset::world_template::WorldTemplate>(asset::world_template::WorldTemplate::NAME)?;

        // Components
        registry.components.define_static::<component::camera::Camera>(component::camera::Camera::NAME)?;
        registry.components.define_static::<component::free_fly::FreeFly>(component::free_fly::FreeFly::NAME)?;
        registry.components.define_static::<component::lifecycle::Lifecycle>(component::lifecycle::Lifecycle::NAME)?;
        registry.components.define_static::<component::static_mesh::StaticMesh>(component::static_mesh::StaticMesh::NAME)?;
        registry.components.define_static::<component::rhai_scripts::RhaiScripts>(component::rhai_scripts::RhaiScripts::NAME)?;
        registry.components.define_static::<component::rigid_body::RigidBody>(component::rigid_body::RigidBody::NAME)?;
        registry.components.define_static::<component::rotator::Rotator>(component::rotator::Rotator::NAME)?;
        registry.components.define_static::<component::script_storage::ScriptStorage>(component::script_storage::ScriptStorage::NAME)?;
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
        registry.systems.define_static("rhai_update_scripts", system::rhai::update_scripts)?;
        registry.systems.define_static("rotator", system::rotator::run)?;
        registry.systems.define_static("transform_propagate", system::transform::propagate)?;
        registry.systems.define_static("ui_update", system::ui::update)?;
        registry.systems.define_static("ui_render", system::ui::render)?;

        Ok(())
    }

    pub fn new(init: SystemCallback) -> Result<Self> {
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
        engine.ecs.setup(init, engine.registry.get_mut())?;
        Ok(engine)
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
        tuple.serialize_element(&AssetManagerSerialize { manager: &self.asset })?;
        tuple.serialize_element(&RendererManagerSerialize { manager: &self.renderer })?;
        tuple.serialize_element(&ECSManagerSerialize { manager: &self.ecs, registry: &self.registry.borrow() })?;
        tuple.serialize_element(&InputManagerSerialize { manager: &self.input })?;
        tuple.serialize_element(&self.accumulator)?;
        tuple.serialize_element(&self.time)?;
        tuple.end()
    }

    pub fn load_state<'de, D: Deserializer<'de>>(&'de mut self, deserializer: D) -> Result<(), D::Error> {
        struct EngineVisitor<'a> {
            engine: &'a mut Engine,
        }
        impl<'de, 'a: 'de> Visitor<'de> for EngineVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Engine")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: serde::de::SeqAccess<'de> {
                use serde::de::Error;
                struct AssetManagerDeserializeSeed<'a> {
                    manager: &'a mut AssetManager,
                    registry: Ref<'a, RegistryManager>,
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
                    registry: Ref<'a, RegistryManager>,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ECSManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(&self.registry, deserializer)
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
                seq.next_element_seed(AssetManagerDeserializeSeed { manager: &mut self.engine.asset, registry: self.engine.registry.borrow() })?;
                seq.next_element_seed(RendererManagerDeserializeSeed { manager: &mut self.engine.renderer })?;
                seq.next_element_seed(ECSManagerDeserializeSeed { manager: &mut self.engine.ecs, registry: self.engine.registry.borrow() })?;
                seq.next_element_seed(InputManagerDeserializeSeed { manager: &mut self.engine.input })?;
                self.engine.accumulator = seq.next_element()?.with_context(|| "Expect accumulator").map_err(Error::custom)?;
                self.engine.time = seq.next_element()?.with_context(|| "Expect time").map_err(Error::custom)?;
                self.engine.renderer.reset(&mut self.engine.ecs).map_err(Error::custom)?;
                Ok(())
            }
        }
        deserializer.deserialize_tuple(6, EngineVisitor { engine: self })?;
        Ok(())
    }

    pub fn iter_input_tables(&self) -> impl Iterator<Item = &InputTable> {
        self.input.iter_tables()
    }

    pub fn iter_input_actions(&self) -> impl Iterator<Item = (&InputAction, &InputActionState)> {
        self.input.iter_actions()
    }

    pub fn iter_input_axis(&self) -> impl Iterator<Item = (&InputAxis, &InputAxisState)> {
        self.input.iter_axis()
    }

    pub fn progress(&mut self, events: &Events, requests: &mut Requests, mut delta_time: f64) -> Result<()> {

        // ================= PREPARE STAGE ================== //

        // Reset graphics state
        self.renderer.prepare()?;

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

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        self.ecs.update(
            &self.registry, 
            &mut self.asset,
            &mut self.input, 
            &mut self.renderer,
            &mut self.script,
            events,
            delta_time, 
            self.time, 
            FIXED_TIMESTEP, 
            fixed_update_count
        )?;

        // ================= REQUESTS STAGE ================= //

        // Check input requests
        if self.input.reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input.reload_input_mapping = false;
        }

        Ok(())
    }

    pub fn update_renderer(&mut self, backend: &mut impl RendererBackend, reset: bool) -> Result<()> {
        if reset {
            backend.reset()?;
            self.renderer.reset(&mut self.ecs)?;
        }
        self.renderer.update_backend(backend, &self.asset, &mut self.ecs)?;
        Ok(())
    }
}