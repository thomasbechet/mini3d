use std::collections::{HashMap, VecDeque};
use core::cell::RefCell;
use anyhow::{Result, Context};
use serde::{Serialize, ser::{SerializeTuple, SerializeSeq}, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{uid::UID, renderer::RendererManager, script::ScriptManager, input::InputManager, asset::AssetManager, registry::{RegistryManager, component::ComponentRegistry}, context::SystemContext};

use self::{world::World, scheduler::Scheduler, procedure::Procedure};

pub mod container;
pub mod entity;
pub mod pipeline;
pub mod procedure;
pub mod query;
pub mod scheduler;
pub mod sparse;
pub mod view;
pub mod world;

#[derive(Default)]
pub struct ECSManager {
    scheduler: RefCell<Scheduler>,
    next_frame_procedures: VecDeque<UID>,
    pub(crate) worlds: RefCell<HashMap<UID, RefCell<Box<World>>>>,
    pub(crate) active_world: UID,
}

impl ECSManager {

    pub(crate) fn save_state<S: Serializer>(&self, registry: &RegistryManager, serializer: S) -> Result<S::Ok, S::Error> {
        struct WorldsSerialize<'a> {
            registry: &'a ComponentRegistry,
            worlds: &'a HashMap<UID, RefCell<Box<World>>>,
        }
        impl<'a> Serialize for WorldsSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                struct WorldSerialize<'a> {
                    registry:  &'a ComponentRegistry,
                    world: &'a World,
                }
                impl<'a> Serialize for WorldSerialize<'a> {
                    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                        self.world.serialize(serializer, self.registry)
                    }
                }
                let mut seq = serializer.serialize_seq(Some(self.worlds.len()))?;
                for world in self.worlds.values() {
                    seq.serialize_element(&WorldSerialize { registry: self.registry, world: &world.borrow() })?;
                }
                seq.end()
            }
        }
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(&self.scheduler);
        tuple.serialize_element(&WorldsSerialize { registry: &registry.components, worlds: &self.worlds.borrow() })?;
        tuple.serialize_element(&self.next_frame_procedures)?;
        tuple.serialize_element(&self.active_world)?;
        tuple.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, registry: &RegistryManager, deserializer: D) -> Result<(), D::Error> {
        struct ECSVisitor<'a> {
            registry: &'a ComponentRegistry,
            manager: &'a mut ECSManager,
        }
        impl<'de, 'a> Visitor<'de> for ECSVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("ECS Manager")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>, {
                use serde::de::Error;
                struct WorldsDeserializeSeed<'a> {
                    registry: &'a ComponentRegistry,
                }
                impl<'a, 'de> DeserializeSeed<'de> for WorldsDeserializeSeed<'a> {
                    type Value = RefCell<HashMap<UID, RefCell<Box<World>>>>;
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct WorldsVisitor<'a> {
                            registry: &'a ComponentRegistry,
                        }
                        impl<'a, 'de> Visitor<'de> for WorldsVisitor<'a> {
                            type Value = RefCell<HashMap<UID, RefCell<Box<World>>>>;
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("Worlds")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: SeqAccess<'de>, {
                                struct WorldDeserializeSeed<'a> {
                                    registry: &'a ComponentRegistry,
                                }
                                impl<'a, 'de> DeserializeSeed<'de> for WorldDeserializeSeed<'a> {
                                    type Value = RefCell<Box<World>>;
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        Ok(RefCell::new(Box::new(World::deserialize(self.registry, deserializer)?)))
                                    }
                                }
                                let mut worlds = HashMap::new();
                                while let Some(world) = seq.next_element_seed(WorldDeserializeSeed { registry: self.registry })? {
                                    let uid: UID = world.borrow().name.as_str().into();
                                    if worlds.contains_key(&uid) {
                                        return Err(A::Error::custom(format!("Duplicate world name: {}", uid)));
                                    }
                                    worlds.insert(uid, world);
                                }
                                Ok(RefCell::new(worlds))
                            }
                        }
                        deserializer.deserialize_seq(WorldsVisitor { registry: self.registry })
                    }
                }
                self.manager.scheduler = seq.next_element()?.with_context(|| "Expect scheduler").map_err(A::Error::custom)?;
                self.manager.worlds = seq.next_element_seed(WorldsDeserializeSeed { registry: self.registry })?.with_context(|| "Expect worlds").map_err(A::Error::custom)?;
                self.manager.next_frame_procedures = seq.next_element()?.with_context(|| "Expect next frame procedures").map_err(A::Error::custom)?;
                self.manager.active_world = seq.next_element()?.with_context(|| "Expect active world").map_err(A::Error::custom)?;
                Ok(())
            }
        }
        self.worlds.borrow_mut().clear();
        self.scheduler = Default::default();
        deserializer.deserialize_tuple(4, ECSVisitor { manager: self, registry: &registry.components })?;
        Ok(())
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
        let change_world: RefCell<Option<UID>> = RefCell::new(None);
    
        // Collect procedures
        let mut frame_procedures = self.next_frame_procedures.drain(..).collect::<VecDeque<_>>();
        for _ in 0..fixed_update_count {
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
                    change_world: &change_world,
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
        if let Some(world) = *change_world.borrow() {
            self.active_world = world;
            self.next_frame_procedures.push_front(Procedure::WORLD_CHANGED.into());
        }

        Ok(())
    }
}