use std::collections::{HashMap, VecDeque};
use core::cell::RefCell;
use anyhow::{Result, Context};
use serde::{Serialize, ser::{SerializeTuple, SerializeSeq}, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{uid::UID, renderer::RendererManager, script::ScriptManager, input::InputManager, asset::AssetManager, registry::RegistryManager, context::{world::WorldContext, SystemContext}};

use self::{world::World, scheduler::Scheduler, procedure::Procedure};

pub mod container;
pub mod entity;
pub mod pipeline;
pub mod procedure;
pub mod query;
pub mod scheduler;
pub mod view;
pub mod world;

#[derive(Default)]
pub struct ECSManager {
    scheduler: RefCell<Scheduler>,
    next_frame_procedures: VecDeque<UID>,
    worlds: RefCell<HashMap<UID, RefCell<Box<World>>>>,
    active_world: UID,
}

impl ECSManager {

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct WorldSerialize<'a> {
            manager: &'a ECSManager,
            world: UID,
        }
        impl<'a> Serialize for WorldSerialize<'a> {
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
                                let schedule: SystemSet = seq.next_element()?.with_context(|| "Expect scene schedule").map_err(Error::custom)?;
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

    pub(crate) fn active_world(&mut self, registry: &RefCell<RegistryManager>) -> WorldContext<'_> {
        WorldContext::new(registry, &mut self.worlds.borrow().get(&self.active_world).unwrap().borrow_mut())
    }

    pub fn update(
        &mut self,
        registry: &RefCell<RegistryManager>,
        asset: &RefCell<AssetManager>,
        input: &RefCell<InputManager>,
        renderer: &RefCell<RendererManager>,
        script: &RefCell<ScriptManager>,
        delta_time: f64,
        time: f64,
        fixed_delta_time: f64,
        fixed_update_count: u32,
    ) -> Result<()> {

        // Prepare frame
        let change_world: Option<UID> = None;
    
        // Collect procedures
        let mut frame_procedures = self.next_frame_procedures.drain(..).collect::<VecDeque<_>>();
        for i in 0..fixed_update_count {
            frame_procedures.push_back(Procedure::FIXED_UPDATE.into());
        }
        frame_procedures.push_back(Procedure::UPDATE.into());

        // Run procedures
        // TODO: protect against infinite loop
        while let Some(procedure) = frame_procedures.pop_front() {

            // Build pipeline
            if let Some(pipeline) = self.scheduler.borrow().build_pipeline(procedure, &registry.borrow().systems)? {
                
                // Build context
                let context = SystemContext {
                    registry,
                    asset,
                    input,
                    renderer,
                    scheduler: &self.scheduler,
                    worlds: &self.worlds,
                    active_world: self.active_world,
                    change_world: &mut change_world,
                    active_procedure: procedure,
                    frame_procedures: &mut frame_procedures,
                    next_frame_procedures: &mut self.next_frame_procedures,
                    delta_time: if procedure == Procedure::FIXED_UPDATE.into() { fixed_delta_time } else { delta_time },
                    time,
                };

                // Run pipeline
                pipeline.run(&context, &script.borrow())?;
            }
        }

        // Change world
        if let Some(world) = change_world {
            self.active_world = world;
            self.next_frame_procedures.push_front(Procedure::WORLD_CHANGED.into());
        }

        Ok(())
    }
}