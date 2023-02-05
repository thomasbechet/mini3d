use anyhow::{Result, Context};
use serde::de::{Visitor, DeserializeSeed};
use serde::ser::SerializeTuple;
use serde::{Serializer, Deserializer, Serialize, Deserialize};

use crate::asset::{AssetManager, AssetEntry};
use crate::feature::{asset, component, system};
use crate::physics::PhysicsManager;
use crate::renderer::RendererManager;
use crate::renderer::backend::RendererBackend;
use crate::scene::{SceneManager, Scene};
use crate::event::Events;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::request::Requests;
use crate::scene::component::Component;
use crate::scene::system::BuiltinSystem;
use crate::script::ScriptManager;
use crate::uid::UID;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct Engine {
    pub(crate) asset: AssetManager,
    pub(crate) input: InputManager,
    pub(crate) script: ScriptManager,
    pub(crate) scene: SceneManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    accumulator: f64,
    time: f64,
}

impl Engine {

    fn define_core_features(&mut self) -> Result<()> {

        // Assets
        self.define_asset::<asset::font::Font>("font")?;
        self.define_asset::<asset::input_action::InputAction>("input_action")?;
        self.define_asset::<asset::input_axis::InputAxis>("input_axis")?;
        self.define_asset::<asset::input_table::InputTable>("input_table")?;
        self.define_asset::<asset::material::Material>("material")?;
        self.define_asset::<asset::mesh::Mesh>("mesh")?;
        self.define_asset::<asset::model::Model>("model")?;
        self.define_asset::<asset::rhai_script::RhaiScript>("rhai_script")?;
        self.define_asset::<asset::schedule::Schedule>("schedule")?;
        self.define_asset::<asset::texture::Texture>("texture")?;
        self.define_asset::<asset::tilemap::Tilemap>("tilemap")?;
        self.define_asset::<asset::tileset::Tileset>("tileset")?;
        self.define_asset::<asset::ui_template::UITemplate>("ui_template")?;
        self.define_asset::<asset::world_template::WorldTemplate>("world_template")?;

        // Components
        self.define_component::<component::camera::Camera>("camera")?;
        self.define_component::<component::free_fly::FreeFly>("free_fly")?;
        self.define_component::<component::lifecycle::Lifecycle>("lifecycle")?;
        self.define_component::<component::model::Model>("model")?;
        self.define_component::<component::rhai_scripts::RhaiScripts>("rhai_scripts")?;
        self.define_component::<component::rigid_body::RigidBody>("rigid_body")?;
        self.define_component::<component::rotator::Rotator>("rotator")?;
        self.define_component::<component::script_storage::ScriptStorage>("script_storage")?;
        self.define_component::<component::transform::Transform>("transform")?;
        self.define_component::<component::transform::LocalToWorld>("local_to_world")?;
        self.define_component::<component::hierarchy::Hierarchy>("hierarchy")?;
        self.define_component::<component::ui::UIComponent>("ui")?;
        self.define_component::<component::viewport::Viewport>("viewport")?;
        self.define_component::<component::canvas::Canvas>("canvas")?;

        // Systems
        self.define_system("despawn_entities", BuiltinSystem::exclusive(system::despawn::run))?;
        self.define_system("renderer", BuiltinSystem::exclusive(system::renderer::despawn_renderer_entities))?;
        self.define_system("free_fly", BuiltinSystem::exclusive(system::free_fly::run))?;
        self.define_system("rhai_update_scripts", BuiltinSystem::exclusive(system::rhai::update_scripts))?;
        self.define_system("rotator", BuiltinSystem::exclusive(system::rotator::run))?;
        self.define_system("transform_propagate", BuiltinSystem::exclusive(system::transform::propagate))?;
        self.define_system("ui_update", BuiltinSystem::exclusive(system::ui::update))?;
        self.define_system("ui_render", BuiltinSystem::exclusive(system::ui::render))?;

        Ok(())
    }

    pub fn new(scene: Scene) -> Result<Self> {
        let mut engine = Self {
            asset: Default::default(), 
            input: Default::default(), 
            script: Default::default(),
            scene: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            accumulator: 0.0,
            time: 0.0,
        };
        engine.define_core_features()?;
        Ok(engine)
    }

    pub fn define_asset<A: Serialize + for<'a> Deserialize<'a> + 'static>(&mut self, name: &str) -> Result<()> {
        self.asset.define::<A>(name)
    }

    pub fn define_component<C: Component>(&mut self, name: &str) -> Result<()> {
        self.scene.define_component::<C>(name)
    }

    pub fn define_system(&mut self, name: &str, system: BuiltinSystem) -> Result<()> {
        self.scene.define_system(name, system)
    }

    pub fn iter_asset<A: 'static>(&'_ self) -> Result<impl Iterator<Item = (&UID, &'_ AssetEntry<A>)>> {
        self.asset.iter::<A>()
    }

    pub fn asset_entry<A: 'static>(&'_ self, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.asset.entry::<A>(uid)
    }

    pub fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            manager: &'a SceneManager,
        }
        impl<'a> Serialize for ECSManagerSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.save_state(serializer)
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
        tuple.serialize_element(&ECSManagerSerialize { manager: &self.scene })?;
        tuple.serialize_element(&InputManagerSerialize { manager: &self.input })?;
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
                }
                impl<'de, 'a> DeserializeSeed<'de> for AssetManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(deserializer)
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
                    manager: &'a mut SceneManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ECSManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(deserializer)
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
                seq.next_element_seed(AssetManagerDeserializeSeed { manager: &mut self.engine.asset })?;
                seq.next_element_seed(RendererManagerDeserializeSeed { manager: &mut self.engine.renderer })?;
                seq.next_element_seed(ECSManagerDeserializeSeed { manager: &mut self.engine.scene })?;
                seq.next_element_seed(InputManagerDeserializeSeed { manager: &mut self.engine.input })?;
                self.engine.accumulator = seq.next_element()?.with_context(|| "Expect accumulator").map_err(Error::custom)?;
                self.engine.time = seq.next_element()?.with_context(|| "Expect time").map_err(Error::custom)?;
                self.engine.renderer.reset(&mut self.engine.scene).map_err(Error::custom)?;
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

        // ================= UPDATE STAGE ================= //

        self.scene.update(&mut self.asset, &mut self.input, &mut self.script, &mut self.renderer, delta_time, self.time)?;

        // ================= FIXED UPDATE STAGE ================= //

        for _ in 0..fixed_update_count {
            // TODO: Process fixed update ...
        }

        // ================= REQUESTS STAGE ================= //

        // Check input requests
        if self.input.reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input.reload_input_mapping = false;
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
            self.renderer.reset(&mut self.scene)?;
        }
        self.renderer.update_backend(backend, &self.asset, &mut self.scene)?;
        Ok(())
    }
}