use std::{any::TypeId, collections::{HashMap, VecDeque}, marker::PhantomData};

use anyhow::{Result, anyhow, Context};
use hecs::{serialize::column::{SerializeContext, DeserializeContext}, Archetype, ColumnBatchType, ColumnBatchBuilder};
use serde::{Serialize, ser::{SerializeTuple, SerializeSeq}, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{uid::UID, feature::asset::schedule::Schedule, renderer::RendererManager, script::ScriptManager, input::InputManager, asset::AssetManager};

use self::{component::{AnyComponent, ComponentEntry, TypeComponent, Component}, context::{SystemContext, SceneProxy, SceneProxyCommand, SignalProxy}, system::{BuiltinSystem, BuiltinSystemEntry}, world::World, signal::{SIGNAL_UPDATE, SIGNAL_SCENE_CHANGED}};

pub mod context;
pub mod query;
pub mod component;
pub mod entity;
pub mod signal;
pub mod system;
pub mod world;

struct SceneInfo {
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

    systems: HashMap<UID, BuiltinSystemEntry>,
    components: HashMap<UID, ComponentEntry>,
    component_type_to_uid: HashMap<TypeId, UID>,

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
                struct WorldSerialize<'a> {
                    manager:  &'a SceneManager,
                    world: &'a World,
                }
                impl<'a> Serialize for WorldSerialize<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where S: Serializer {
                        hecs::serialize::column::serialize(&self.world.raw_world, &mut SceneSerializeContext { manager: self.manager, components: Default::default() }, serializer)
                    }
                }
                let data = self.manager.scenes.get(&self.scene).unwrap();
                let info = self.manager.scene_info.get(&self.scene).unwrap();
                let mut tuple = serializer.serialize_tuple(4)?;
                tuple.serialize_element(&info.name)?;
                tuple.serialize_element(&info.index)?;
                tuple.serialize_element(&data.schedule)?;
                tuple.serialize_element(&WorldSerialize { manager: self.manager, world: &data.world })?;
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
                                struct WorldDeserializeSeed<'a> {
                                    manager: &'a SceneManager,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for WorldDeserializeSeed<'a> {
                                    type Value = World;
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        let raw_world: hecs::World = hecs::serialize::column::deserialize(&mut SceneDeserializeContext { manager: self.manager, components: Default::default() }, deserializer)?;
                                        Ok(World { raw_world })
                                    }
                                }
                                use serde::de::Error;
                                let name: String = seq.next_element()?.with_context(|| "Expect scene name").map_err(Error::custom)?;
                                let index: u32 = seq.next_element()?.with_context(|| "Expect scene index").map_err(Error::custom)?;
                                let schedule: Schedule = seq.next_element()?.with_context(|| "Expect scene schedule").map_err(Error::custom)?;
                                let world: World = seq.next_element_seed(WorldDeserializeSeed { manager: self.manager })?.with_context(|| "Expect scene world").map_err(Error::custom)?;
                                let uid = UID::new(&name);
                                if self.manager.scenes.contains_key(&uid) { return Err(Error::custom(format!("Scene world '{}' already exists", name))); }
                                self.manager.scenes.insert(uid, Box::new(Scene { world, schedule }));
                                self.manager.scene_info.insert(uid, SceneInfo { name, index });
                                Ok(())
                            }
                        }
                        deserializer.deserialize_tuple(4, ECSVisitor { manager: self.manager })
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

    pub(crate) fn define_system(&mut self, name: &str, system: BuiltinSystem) -> Result<()> {
        let uid: UID = name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System '{}' already defined", name));
        }
        self.systems.insert(uid, BuiltinSystemEntry { 
            name: name.to_string(),
            callback: system,
        });
        Ok(())
    }

    pub(crate) fn define_component<C: Component>(&mut self, name: &str) -> Result<()> {
        let uid: UID = name.into();
        let type_id = TypeId::of::<C>();
        if self.components.contains_key(&uid) {
            return Err(anyhow!("Component with name '{}' already defined", name));
        }
        if let Some(uid) = self.component_type_to_uid.get(&type_id) {
            let component = self.components.get(uid).unwrap();
            return Err(anyhow!("Component '{}' defined with the same type id", component.name));
        }
        self.components.insert(uid, ComponentEntry { name: name.to_string(), type_id, component: Box::new(TypeComponent::<C> { marker: PhantomData }) });
        self.component_type_to_uid.insert(type_id, uid);
        Ok(())
    }

    pub(crate) fn iter_worlds_mut(&mut self) -> impl Iterator<Item = &mut World> {
        self.scenes.values_mut().map(|scene| &mut scene.world)
    }

    pub fn update(
        &mut self,
        asset: &mut AssetManager,
        input: &mut InputManager,
        script: &mut ScriptManager,
        renderer: &mut RendererManager,
        delta_time: f64,
        time: f64,
    ) -> Result<()> {

        // Extract previous signals
        let signals = self.signal_queue.drain(..).collect::<Vec<_>>();

        // Build context
        let mut commands = Default::default();
        let mut scene_proxy = SceneProxy {
            current: self.active_scene,
            scene_info: &mut self.scene_info,
            commands: &mut commands,
        };
        let mut signal_proxy = SignalProxy {
            signal_queue: &mut self.signal_queue,
        };
        let mut context = SystemContext {
            asset,
            input,
            script,
            renderer,
            scene: &mut scene_proxy,
            signal: &mut signal_proxy,
            delta_time,
            time,
        };
        let scene = self.scenes.get_mut(&self.active_scene).unwrap();
        
        // Invoke signals
        for signal in signals {
            scene.schedule.invoke(signal, &self.systems, &mut context, &mut scene.world)?;
        }

        // Update
        scene.schedule.invoke(SIGNAL_UPDATE.into(), &self.systems, &mut context, &mut scene.world)?;

        // Process commands
        for command in commands {
            match command {
                SceneProxyCommand::Change(uid) => {
                    self.active_scene = uid;
                    self.signal_queue.push_back(SIGNAL_SCENE_CHANGED.into());
                },
                SceneProxyCommand::Load(uid, scene) => {
                    self.scenes.insert(uid, scene);
                },
                SceneProxyCommand::Unload(uid) => {
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

struct SceneSerializeContext<'a> {
    manager: &'a SceneManager,
    components: Vec<UID>,
}

impl<'a> SerializeContext for SceneSerializeContext<'a> {

    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype.component_types()
            .filter(|&t| self.manager.component_type_to_uid.contains_key(&t))
            .count()
    }

    fn serialize_component_ids<S: SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        mut out: S,
    ) -> Result<S::Ok, S::Error> {
        let mut uid_list = self.manager.components.keys().copied().collect::<Vec<_>>();
        uid_list.sort();
        self.components.clear();
        for uid in &uid_list {
            let component = self.manager.components.get(uid).unwrap();
            if archetype.has_dynamic(component.type_id) {
                out.serialize_element(&uid)?;
                self.components.push(*uid);
            }
        }
        out.end()
    }

    fn serialize_components<S: SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        mut out: S,
    ) -> Result<S::Ok, S::Error> {
        struct ArchetypeColumnSerialize<'a> {
            component: &'a dyn AnyComponent,
            archetype: &'a Archetype,
            component_count: u32,
        }
        impl<'a> Serialize for ArchetypeColumnSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer 
            {
                let mut iterator = self.component.serialize_column(self.archetype)
                    .with_context(|| "Failed to create iterator").map_err(S::Error::custom)?;
                let mut tuple = serializer.serialize_tuple(self.component_count as usize)?;
                while let Some(c) = iterator.next() {
                    tuple.serialize_element(c)?;
                }
                tuple.end()
            }
        }
        use serde::ser::Error;
        for uid in &self.components {
            let component = self.manager.components.get(uid)
                .with_context(|| "Component not found").map_err(S::Error::custom)?;
            out.serialize_element(&ArchetypeColumnSerialize { component: component.component.as_ref(), archetype, component_count: archetype.len() })?;
        }
        out.end()
    }
}

struct SceneDeserializeContext<'a> {
    manager: &'a SceneManager,
    components: Vec<UID>,
}

impl<'a> DeserializeContext for SceneDeserializeContext<'a> {

    fn deserialize_component_ids<'de, A>(&mut self, mut seq: A) -> Result<ColumnBatchType, A::Error>
        where A: SeqAccess<'de> 
    {
        use serde::de::Error;
        self.components.clear();
        while let Some(uid) = seq.next_element::<UID>()? {
            self.components.push(uid);
        }
        self.components.sort();
        let mut batch = ColumnBatchType::new();
        for uid in &self.components {
            let component = self.manager.components.get(uid)
                .with_context(|| "Component not found").map_err(A::Error::custom)?;
            component.component.add_to_batch(&mut batch);
        }
        Ok(batch)
    }

    fn deserialize_components<'de, A>(
        &mut self,
        entity_count: u32,
        mut seq: A,
        batch: &mut ColumnBatchBuilder,
    ) -> Result<(), A::Error>
        where A: SeqAccess<'de> 
    {
        struct ArchetypeColumnDeserializeSeed<'a> {
            component: &'a ComponentEntry,
            batch: &'a mut ColumnBatchBuilder,
            entity_count: u32,
        }
        impl<'de, 'a> DeserializeSeed<'de> for ArchetypeColumnDeserializeSeed<'a> {
            type Value = ();
            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where D: serde::Deserializer<'de> 
            {
                let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
                self.component.component.deserialize_column(self.batch, self.entity_count, &mut deserializer)
                    .map_err(D::Error::custom)
            }
        }
        use serde::de::Error;
        for uid in &self.components {                
            let component = self.manager.components.get(uid)
                .with_context(|| "Component not found").map_err(A::Error::custom)?;
            seq.next_element_seed(ArchetypeColumnDeserializeSeed { component, batch, entity_count })?;
        }
        Ok(())
    }
}