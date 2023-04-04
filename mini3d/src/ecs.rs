use std::collections::{HashMap, VecDeque, HashSet};
use core::cell::RefCell;
use anyhow::{Result, Context};
use serde::{Serialize, ser::{SerializeTuple, SerializeSeq}, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{uid::UID, renderer::RendererManager, script::ScriptManager, input::InputManager, asset::AssetManager, registry::{RegistryManager, component::ComponentRegistry}, context::{SystemContext, asset::AssetContext, input::InputContext, procedure::ProcedureContext, renderer::RendererContext, scheduler::SchedulerContext, world::WorldContext, registry::RegistryContext, time::TimeContext, event::EventContext}, event::Events};

use self::{world::World, scheduler::Scheduler, procedure::Procedure, pipeline::CompiledSystemPipeline};

pub mod reference;
pub mod container;
pub mod entity;
pub mod pipeline;
pub mod procedure;
pub mod query;
pub mod dynamic;
pub mod scheduler;
pub mod singleton;
pub mod sparse;
pub mod system;
pub mod view;
pub mod world;

pub(crate) struct ECSManager {
    scheduler: Scheduler,
    next_frame_system_invocations: Vec<UID>,
    next_frame_procedures: VecDeque<UID>,
    pub(crate) worlds: RefCell<HashMap<UID, RefCell<Box<World>>>>,
    pub(crate) active_world: UID,
}

impl Default for ECSManager {

    fn default() -> Self {
        Self {
            scheduler: Scheduler::default(),
            next_frame_system_invocations: Vec::new(),
            next_frame_procedures: VecDeque::new(),
            worlds: RefCell::new(HashMap::from([(Self::MAIN_WORLD.into(), RefCell::new(Box::new(World::new(Self::MAIN_WORLD))))])),
            active_world: Self::MAIN_WORLD.into(),
        }
    }
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) asset: &'a mut AssetManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) events: &'a Events,
    pub(crate) delta_time: f64,
    pub(crate) time: f64,
    pub(crate) fixed_delta_time: f64,
}

#[allow(clippy::too_many_arguments)]
fn create_system_context<'b, 'a: 'b>(
    context: &'b mut ECSUpdateContext<'a>,
    active_procedure: UID,
    active_world: UID,
    scheduler: &'b mut Scheduler,
    frame_procedures: &'b mut VecDeque<UID>,
    next_frame_procedures: &'b mut VecDeque<UID>,
    worlds: &'b mut HashMap<UID, RefCell<Box<World>>>,
    change_world: &'b mut Option<UID>,
    removed_worlds: &'b mut HashSet<UID>,
) -> SystemContext<'b> {
    SystemContext {
        asset: AssetContext {
            registry: context.registry,
            manager: context.asset,
        },
        event: EventContext {
            events: context.events,
        },
        input: InputContext {
            manager: context.input,
        },
        procedure: ProcedureContext {
            active_procedure,
            frame_procedures,
            next_frame_procedures,
        },
        registry: RegistryContext {
            manager: context.registry,
        },
        renderer: RendererContext {
            manager: context.renderer,
        },
        scheduler: SchedulerContext {
            scheduler,
        },
        time: TimeContext {
            delta: if active_procedure == Procedure::FIXED_UPDATE.into() { context.fixed_delta_time } else { context.delta_time },
            global: context.time,
        },
        world: WorldContext {
            registry: context.registry,
            worlds,
            active_world,
            change_world,
            removed_worlds,
        },
    }
}

impl ECSManager {

    const MAIN_WORLD: &'static str = "main";

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
        tuple.serialize_element(&self.scheduler)?;
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

    pub(crate) fn invoke(&mut self, system: UID) -> Result<()> {
        self.next_frame_system_invocations.push(system);
        Ok(())
    }

    pub(crate) fn update(
        &mut self,
        mut context: ECSUpdateContext,
        scripts: &mut ScriptManager,
        fixed_update_count: u32,
    ) -> Result<()> {

        // Prepare frame
        let mut change_world: Option<UID> = None;
        let mut removed_worlds: HashSet<UID> = Default::default();
        let mut worlds = self.worlds.borrow_mut();

        // Collect procedures
        let mut frame_procedures = self.next_frame_procedures.drain(..).collect::<VecDeque<_>>();
        for _ in 0..fixed_update_count {
            frame_procedures.push_back(Procedure::FIXED_UPDATE.into());
        }
        frame_procedures.push_back(Procedure::UPDATE.into());

        // Invoke frame systems
        if !self.next_frame_system_invocations.is_empty() {
            // Build context
            let pipeline = CompiledSystemPipeline::build(&context.registry.borrow_mut().systems, self.next_frame_system_invocations.iter())?;
            pipeline.run(&mut create_system_context(
                &mut context,
                UID::null(), 
                self.active_world,
                &mut self.scheduler,
                &mut frame_procedures,
                &mut self.next_frame_procedures, 
                &mut worlds,
                &mut change_world,
                &mut removed_worlds,
            ), scripts)?;
            self.next_frame_system_invocations.clear();
        }

        // Run procedures
        // TODO: protect against infinite loop
        while let Some(procedure) = frame_procedures.pop_front() {

            // Build pipeline
            if let Some(pipeline) = self.scheduler.build_pipeline(procedure, context.registry)? {

                // Run pipeline
                pipeline.run(&mut create_system_context(
                    &mut context,
                    procedure, 
                    self.active_world,
                    &mut self.scheduler,
                    &mut frame_procedures,
                    &mut self.next_frame_procedures, 
                    &mut worlds,
                    &mut change_world,
                    &mut removed_worlds
                ), scripts)?;
            }

            // Remove worlds
            for uid in removed_worlds.drain() {
                self.worlds.borrow_mut().remove(&uid);
            }

            // Change world
            if let Some(world) = change_world {
                self.active_world = world;
                self.next_frame_procedures.push_front(Procedure::WORLD_CHANGED.into());
                change_world = None;
            }
        }

        Ok(())
    }
}