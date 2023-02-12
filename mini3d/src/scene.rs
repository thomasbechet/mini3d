use std::collections::{HashMap, VecDeque};
use core::cell::RefCell;
use anyhow::{Result, Context};
use serde::{Serialize, ser::{SerializeTuple, SerializeSeq}, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{uid::UID, feature::asset::schedule::Schedule, renderer::RendererManager, script::ScriptManager, input::InputManager, asset::AssetManager, registry::RegistryManager, context::{SystemContext, scene::SceneCommand}};

use self::{world::World, signal::{SIGNAL_UPDATE, SIGNAL_SCENE_CHANGED}};

pub mod query;
pub mod container;
pub mod entity;
pub mod signal;
pub mod view;
pub mod world;

pub(crate) struct SceneInfo {
    name: String,
    index: u32,
}

pub struct Scene {
    schedule: Schedule,
    world: World,
}

impl Scene {
    pub fn new(world: World, schedule: Schedule) -> Self {
        Self { world, schedule }
    }
}

#[derive(Default)]
pub struct SceneManager {
    scene_info: HashMap<UID, SceneInfo>,
    scenes: HashMap<UID, Box<Scene>>,
    active_scene: UID,
    signal_queue: VecDeque<UID>,
}

impl SceneManager {

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct SceneSerialize<'a> {
            manager: &'a SceneManager,
            scene: UID,
        }
        impl<'a> Serialize for SceneSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                // struct WorldSerialize<'a> {
                //     manager:  &'a SceneManager,
                //     world: &'a World,
                // }
                // impl<'a> Serialize for WorldSerialize<'a> {
                //     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                //         where S: Serializer {
                //         hecs::serialize::column::serialize(&self.world.raw_world, &mut SceneSerializeContext { manager: self.manager, components: Default::default() }, serializer)
                //     }
                // }
                let data = self.manager.scenes.get(&self.scene).unwrap();
                let info = self.manager.scene_info.get(&self.scene).unwrap();
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&info.name)?;
                tuple.serialize_element(&info.index)?;
                tuple.serialize_element(&data.schedule)?;
                // tuple.serialize_element(&WorldSerialize { manager: self.manager, world: &data.world })?;
                tuple.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.scenes.len()))?;
        for uid in self.scenes.keys().collect::<Vec<_>>() {
            seq.serialize_element(&SceneSerialize { manager: self, scene: *uid })?;
        }
        seq.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct SceneVisitor<'a> {
            manager: &'a mut SceneManager,
        }
        impl<'de, 'a> Visitor<'de> for SceneVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Sequence of scene")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>, {
                struct SceneDeserializeSeed<'a> {
                    manager: &'a mut SceneManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for SceneDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct ECSVisitor<'a> {
                            manager: &'a mut SceneManager,
                        }
                        impl<'de, 'a> Visitor<'de> for ECSVisitor<'a> {
                            type Value = ();
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("Scene")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: SeqAccess<'de> {
                                // struct WorldDeserializeSeed<'a> {
                                //     manager: &'a SceneManager,
                                // }
                                // impl<'de, 'a> DeserializeSeed<'de> for WorldDeserializeSeed<'a> {
                                //     type Value = World;
                                //     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                //         where D: Deserializer<'de> {
                                //         let raw_world: hecs::World = hecs::serialize::column::deserialize(&mut SceneDeserializeContext { manager: self.manager, components: Default::default() }, deserializer)?;
                                //         Ok(World { raw_world })
                                //     }
                                // }
                                use serde::de::Error;
                                let name: String = seq.next_element()?.with_context(|| "Expect scene name").map_err(Error::custom)?;
                                let index: u32 = seq.next_element()?.with_context(|| "Expect scene index").map_err(Error::custom)?;
                                let schedule: Schedule = seq.next_element()?.with_context(|| "Expect scene schedule").map_err(Error::custom)?;
                                // let world: World = seq.next_element_seed(WorldDeserializeSeed { manager: self.manager })?.with_context(|| "Expect scene world").map_err(Error::custom)?;
                                let uid = UID::new(&name);
                                if self.manager.scenes.contains_key(&uid) { return Err(Error::custom(format!("Scene world '{}' already exists", name))); }
                                // self.manager.scenes.insert(uid, Box::new(Scene { world, schedule }));
                                self.manager.scene_info.insert(uid, SceneInfo { name, index });
                                Ok(())
                            }
                        }
                        deserializer.deserialize_tuple(3, ECSVisitor { manager: self.manager })
                    }
                }
                while seq.next_element_seed(SceneDeserializeSeed { manager: self.manager })?.is_some() {}
                Ok(())
            }
        }
        self.scenes.clear();
        self.scene_info.clear();
        deserializer.deserialize_seq(SceneVisitor { manager: self })?;
        Ok(())
    }

    pub(crate) fn iter_worlds_mut(&mut self) -> impl Iterator<Item = &mut World> {
        self.scenes.values_mut().map(|scene| &mut scene.world)
    }

    pub fn update(
        &mut self,
        registry: &RefCell<RegistryManager>,
        asset: &RefCell<AssetManager>,
        input: &RefCell<InputManager>,
        script: &RefCell<ScriptManager>,
        renderer: &RefCell<RendererManager>,
        delta_time: f64,
        time: f64,
    ) -> Result<()> {

        // Prepare signals and commands
        let signals = self.signal_queue.drain(..).collect::<Vec<_>>();
        let mut commands: Vec<SceneCommand> = Default::default();
        let scene = self.scenes.get_mut(&self.active_scene).unwrap();

        // Build context
        let mut context = SystemContext {
            registry,
            asset,
            input,
            renderer,
            world: &mut scene.world,
            delta_time,
            time,
            active_scene: self.active_scene,
            scene_info: &self.scene_info,
            scene_commands: &mut commands,
            signal_queue: &mut self.signal_queue,
        };

        // Invoke signals
        for signal in signals {
            scene.schedule.invoke(signal, &mut context)?;
        }

        // Update
        scene.schedule.invoke(SIGNAL_UPDATE.into(), &mut context)?;

        // Process commands
        for command in commands {
            match command {
                SceneCommand::Change(uid) => {
                    self.active_scene = uid;
                    self.signal_queue.push_back(SIGNAL_SCENE_CHANGED.into());
                },
                SceneCommand::Load(uid, scene) => {
                    self.scenes.insert(uid, scene);
                },
                SceneCommand::Unload(uid) => {
                    self.scenes.remove(&uid);
                },
            }
        }

        Ok(())
    }

    pub fn fixed_update(&mut self, delta_time: f32) -> Result<()> {
        // TODO: invoke fixed update
        Ok(())
    }
}