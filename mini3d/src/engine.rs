use anyhow::{Result, Context};
use serde::de::{Visitor, DeserializeSeed};
use serde::ser::SerializeTuple;
use serde::{Serializer, Deserializer, Serialize};

use crate::asset::AssetManager;
use crate::feature::{asset, component, system, signal, process};
use crate::renderer::RendererManager;
use crate::renderer::backend::RendererBackend;
use crate::scene::SceneManager;
use crate::event::Events;
use crate::event::system::SystemEvent;
use crate::input::InputManager;
use crate::process::{ProcessManager, ProcessManagerContext};
use crate::request::Requests;
use crate::script::ScriptManager;
use crate::signal::SignalManager;

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;
const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

pub struct Engine {
    pub asset: AssetManager,
    pub input: InputManager,
    pub process: ProcessManager,
    pub script: ScriptManager,
    pub scene: SceneManager,
    pub signal: SignalManager,
    pub renderer: RendererManager,
    accumulator: f64,
    time: f64,
}

impl Engine {

    fn register_feature(&mut self) -> Result<()> {

        // Assets
        self.asset.register::<asset::font::FontAsset>("font")?;
        self.asset.register::<asset::input_action::InputActionAsset>("input_action")?;
        self.asset.register::<asset::input_axis::InputAxisAsset>("input_axis")?;
        self.asset.register::<asset::input_table::InputTableAsset>("input_table")?;
        self.asset.register::<asset::material::MaterialAsset>("material")?;
        self.asset.register::<asset::mesh::MeshAsset>("mesh")?;
        self.asset.register::<asset::model::ModelAsset>("model")?;
        self.asset.register::<asset::rhai_script::RhaiScriptAsset>("rhai_script")?;
        self.asset.register::<asset::scene::SceneAsset>("scene")?;
        self.asset.register::<asset::system_schedule::SystemScheduleAsset>("system_schedule")?;
        self.asset.register::<asset::texture::TextureAsset>("texture")?;
        self.asset.register::<asset::tilemap::TilemapAsset>("tilemap")?;
        self.asset.register::<asset::tileset::TilesetAsset>("tileset")?;
        self.asset.register::<asset::ui::UIAsset>("ui")?;

        // Components
        self.scene.register_component::<component::camera::CameraComponent>("camera")?;
        self.scene.register_component::<component::free_fly::FreeFlyComponent>("free_fly")?;
        self.scene.register_component::<component::lifecycle::LifecycleComponent>("lifecycle")?;
        self.scene.register_component::<component::model::ModelComponent>("model")?;
        self.scene.register_component::<component::rhai_scripts::RhaiScriptsComponent>("rhai_scripts")?;
        self.scene.register_component::<component::rotator::RotatorComponent>("rotator")?;
        self.scene.register_component::<component::script_storage::ScriptStorageComponent>("script_storage")?;
        self.scene.register_component::<component::transform::TransformComponent>("transform")?;
        self.scene.register_component::<component::transform::LocalToWorldComponent>("local_to_world")?;
        self.scene.register_component::<component::hierarchy::HierarchyComponent>("hierarchy")?;
        self.scene.register_component::<component::ui::UIComponent>("ui")?;
        self.scene.register_component::<component::viewport::ViewportComponent>("viewport")?;
        self.scene.register_component::<component::canvas::CanvasComponent>("canvas")?;

        // Processes
        self.process.register::<process::profiler::ProfilerProcess>("profiler")?;

        // Systems
        self.scene.register_system("despawn_entities", system::despawn::run)?;
        self.scene.register_system("free_fly", system::free_fly::run)?;
        self.scene.register_system("renderer", system::renderer::despawn_renderer_entities)?;
        self.scene.register_system("rhai_update_scripts", system::rhai::update_scripts)?;
        self.scene.register_system("rotator", system::rotator::run)?;
        self.scene.register_system("transform_propagate", system::transform::propagate)?;
        self.scene.register_system("ui_update", system::ui::update)?;
        self.scene.register_system("ui_render", system::ui::render)?;

        // Signals
        self.signal.register::<signal::command::CommandSignal>("command")?;

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let mut engine = Self {
            asset: Default::default(), 
            input: Default::default(), 
            process: Default::default(),
            script: Default::default(),
            scene: Default::default(),
            signal: Default::default(),
            renderer: Default::default(),
            accumulator: 0.0,
            time: 0.0,
        };
        engine.register_feature()?;
        Ok(engine)
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
        struct ProcessManagerSerialize<'a> {
            manager: &'a ProcessManager,
        }
        impl<'a> Serialize for ProcessManagerSerialize<'a> {
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
        struct SignalManagerSerialize<'a> {
            manager: &'a SignalManager,
        }
        impl<'a> Serialize for SignalManagerSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.save_state(serializer)
            }
        }
        let mut tuple = serializer.serialize_tuple(8)?;
        tuple.serialize_element(&AssetManagerSerialize { manager: &self.asset })?;
        tuple.serialize_element(&RendererManagerSerialize { manager: &self.renderer })?;
        tuple.serialize_element(&ProcessManagerSerialize { manager: &self.process })?;
        tuple.serialize_element(&ECSManagerSerialize { manager: &self.scene })?;
        tuple.serialize_element(&InputManagerSerialize { manager: &self.input })?;
        tuple.serialize_element(&SignalManagerSerialize { manager: &self.signal })?;
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
                struct ProcessManagerDeserializeSeed<'a> {
                    manager: &'a mut ProcessManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ProcessManagerDeserializeSeed<'a> {
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
                struct SignalManagerDeserializeSeed<'a> {
                    manager: &'a mut SignalManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for SignalManagerDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.load_state(deserializer)
                    }
                }
                seq.next_element_seed(AssetManagerDeserializeSeed { manager: &mut self.engine.asset })?;
                seq.next_element_seed(RendererManagerDeserializeSeed { manager: &mut self.engine.renderer })?;
                seq.next_element_seed(ProcessManagerDeserializeSeed { manager: &mut self.engine.process })?;
                seq.next_element_seed(ECSManagerDeserializeSeed { manager: &mut self.engine.scene })?;
                seq.next_element_seed(InputManagerDeserializeSeed { manager: &mut self.engine.input })?;
                seq.next_element_seed(SignalManagerDeserializeSeed { manager: &mut self.engine.signal })?;
                self.engine.accumulator = seq.next_element()?.with_context(|| "Expect accumulator").map_err(Error::custom)?;
                self.engine.time = seq.next_element()?.with_context(|| "Expect time").map_err(Error::custom)?;
                self.engine.renderer.reset(&mut self.engine.scene).map_err(Error::custom)?;
                Ok(())
            }
        }
        deserializer.deserialize_tuple(8, EngineVisitor { engine: self })?;
        Ok(())
    }

    pub fn progress(
        &mut self,
        events: &Events,
        requests: &mut Requests,
        mut delta_time: f64,
    ) -> Result<()> {

        // ================= PREPARE STEP ================== //
        self.renderer.prepare()?;

        // ================= DISPATCH STEP ================= //

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

        // ================= UPDATE STEP ================= //

        // Compute accumulated time since last progress
        if delta_time > MAXIMUM_TIMESTEP {
            delta_time = MAXIMUM_TIMESTEP; // Slowing down
        }
        self.accumulator += delta_time;
        self.time += delta_time;

        // Update processes
        let mut ctx = ProcessManagerContext {
            asset: &mut self.asset,
            input: &mut self.input,
            script: &mut self.script,
            scene: &mut self.scene,
            signal: &mut self.signal,
            renderer: &mut self.renderer,
            events,
            delta_time,
            time: self.time,
        };
        self.process.update(&mut ctx)?;

        // ================= FIXED UPDATE STEP ================= //

        // delta_time = FIXED_TIMESTEP;
        while self.accumulator >= FIXED_TIMESTEP {
            self.accumulator -= FIXED_TIMESTEP;

            // Process fixed update
        }

        // ================= REQUESTS STEP ================= //

        // Check input requests
        if self.input.reload_input_mapping {
            requests.reload_input_mapping = true;
            self.input.reload_input_mapping = false;
        }

        // ================= CLEANUP STEP ================= // 
        self.signal.cleanup();

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