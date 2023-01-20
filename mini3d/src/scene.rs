use std::{any::{TypeId, type_name}, collections::HashMap, marker::PhantomData};

use anyhow::{Result, anyhow, Context};
use hecs::{World, serialize::column::{SerializeContext, DeserializeContext}, Archetype, ColumnBatchType, ColumnBatchBuilder, ArchetypeColumn, Entity};
use serde::{Serialize, Deserialize, ser::{SerializeTuple, SerializeSeq}, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{asset::AssetManager, input::InputManager, script::ScriptManager, uid::UID, process::ProcessContext, feature::asset::system_schedule::{SystemScheduleType, SystemScheduleAsset}, signal::SignalManager, renderer::RendererManager};

pub mod component;
pub mod entity;
pub mod world;

pub struct SystemContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub signal: &'a mut SignalManager,
    pub script: &'a mut ScriptManager,
    pub renderer: &'a mut RendererManager,
    pub delta_time: f64,
    pub time: f64,
    pub scene_uid: UID,
}

pub type SystemRunCallback = fn(&mut SystemContext, &mut World) -> Result<()>;

struct SystemCallbacks {
    run: SystemRunCallback,
}

struct SystemEntry {
    name: String,
    callbacks: SystemCallbacks,
}

trait AnyComponent {
    fn serialize_column<'a>(&'a self, archetype: &'a Archetype) -> Result<Box<dyn AnyArchetypeColumnIterator<'a> + 'a>>;
    fn deserialize_column(&self, batch: &mut ColumnBatchBuilder, entity_count: u32, deserializer: &mut dyn erased_serde::Deserializer) -> Result<()>;
    fn add_to_batch(&self, batch: &mut ColumnBatchType);
}

struct Component<C> { marker: PhantomData<C> }

trait AnyComponentDeserializeSeed<'a> {}

trait AnyArchetypeColumnIterator<'a> {
    fn next<'b>(&'b mut self) -> Option<&'b (dyn erased_serde::Serialize + 'b)>;
}

impl<C: hecs::Component + Serialize + for<'de> Deserialize<'de>> AnyComponent for Component<C> {
    fn serialize_column<'a>(&'a self, archetype: &'a Archetype) -> Result<Box<dyn AnyArchetypeColumnIterator<'a> + 'a>> {
        struct ArchetypeColumnIterator<'a, C: hecs::Component + Serialize> {
            reference: ArchetypeColumn<'a, C>,
            next: usize,
        }
        impl<'a, C: hecs::Component + Serialize> AnyArchetypeColumnIterator<'a> for ArchetypeColumnIterator<'a, C> {
            fn next<'b>(&'b mut self) -> Option<&'b (dyn erased_serde::Serialize + 'b)> {
                let current = self.next;
                self.next += 1;
                match self.reference.get(current) {
                    Some(r) => Some(r),
                    None => None,
                }
            }
        }
        let reference = archetype.get::<&C>().with_context(|| "Archetype doesn't contain component")?;
        Ok(Box::new(ArchetypeColumnIterator { reference, next: 0 }))
    }
    fn deserialize_column(&self, batch: &mut ColumnBatchBuilder, entity_count: u32, deserializer: &mut dyn erased_serde::Deserializer) -> Result<()> {
        struct ColumnVisitor<'a, C: hecs::Component + for<'de> Deserialize<'de>> {
            batch: &'a mut ColumnBatchBuilder,
            entity_count: u32,
            marker: PhantomData<C>,
        }
        impl<'de, 'a, C: hecs::Component + for<'b> Deserialize<'b>> Visitor<'de> for ColumnVisitor<'a, C> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a set of {} {} values", self.entity_count, type_name::<C>())
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>,
            {
                use serde::de::Error;
                let mut out = self.batch.writer::<C>().expect("Unexpected component type");
                while let Some(component) = seq.next_element()? {
                    if out.push(component).is_err() {
                        return Err(A::Error::custom("Extra component"));
                    }
                }
                if out.fill() < self.entity_count {
                    return Err(A::Error::custom("Invalid tuple length"));
                }
                Ok(())
            }
        }
        deserializer.deserialize_tuple(entity_count as usize, ColumnVisitor::<C> { batch, entity_count, marker: PhantomData })?;
        Ok(())
    }
    fn add_to_batch(&self, batch: &mut ColumnBatchType) {
        batch.add::<C>();
    }
}

trait AnyArchetypeColumnSerialize {
    fn serialize_column(&self) -> Vec<&dyn erased_serde::Serialize>;
}

struct ComponentEntry {
    name: String,
    type_id: TypeId,
    component: Box<dyn AnyComponent>,
}

#[derive(Default, Serialize, Deserialize)]
struct SystemScheduler {
    systems: Vec<SystemScheduleType>,
}

struct SceneInstance {
    name: String,
    world: World,
    scheduler: SystemScheduler,
}

#[derive(Default)]
pub struct SceneManager {
    systems: HashMap<UID, SystemEntry>,
    components: HashMap<UID, ComponentEntry>,
    component_type_to_uid: HashMap<TypeId, UID>,
    instances: HashMap<UID, SceneInstance>,
}

impl SceneManager {

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct ECSSerialize<'a> {
            manager: &'a SceneManager,
            instance: &'a SceneInstance,
        }
        impl<'a> Serialize for ECSSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                struct WorldSerialize<'a> {
                    manager:  &'a SceneManager,
                    world: &'a World,
                }
                impl<'a> Serialize for WorldSerialize<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where S: Serializer {
                        hecs::serialize::column::serialize(self.world, &mut SceneSerializeContext { manager: self.manager, components: Default::default() }, serializer)
                    }
                }
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&self.instance.name)?;
                tuple.serialize_element(&self.instance.scheduler)?;
                tuple.serialize_element(&WorldSerialize { manager: self.manager, world: &self.instance.world })?;
                tuple.end()
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.instances.len()))?;
        for instance in self.instances.values() {
            seq.serialize_element(&ECSSerialize { manager: self, instance })?;
        }
        seq.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct ECSVisitor<'a> {
            manager: &'a mut SceneManager,
        }
        impl<'de, 'a> Visitor<'de> for ECSVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Sequence of ECS")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>, {
                struct ECSDeserializeSeed<'a> {
                    manager: &'a mut SceneManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ECSDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct ECSVisitor<'a> {
                            manager: &'a mut SceneManager,
                        }
                        impl<'de, 'a> Visitor<'de> for ECSVisitor<'a> {
                            type Value = ();
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("ECS")
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
                                        hecs::serialize::column::deserialize(&mut SceneDeserializeContext { manager: self.manager, components: Default::default() }, deserializer)
                                    }
                                }
                                use serde::de::Error;
                                let name: String = seq.next_element()?.with_context(|| "Expect ECS name").map_err(Error::custom)?;
                                let scheduler: SystemScheduler = seq.next_element()?.with_context(|| "Expect ECS system scheduler").map_err(Error::custom)?;
                                let world: World = seq.next_element_seed(WorldDeserializeSeed { manager: self.manager })?.with_context(|| "Expect ECS world").map_err(Error::custom)?;
                                let uid = UID::new(&name);
                                if self.manager.instances.contains_key(&uid) { return Err(Error::custom(format!("ECS world '{}' already exists", name))); }
                                self.manager.instances.insert(uid, SceneInstance { name, world, scheduler });
                                Ok(())
                            }
                        }
                        deserializer.deserialize_tuple(3, ECSVisitor { manager: self.manager })
                    }
                }
                while seq.next_element_seed(ECSDeserializeSeed { manager: self.manager })?.is_some() {}
                Ok(())
            }
        }
        self.instances.clear();
        deserializer.deserialize_seq(ECSVisitor { manager: self })?;
        Ok(())
    }

    pub(crate) fn iter_world(&'_ mut self) -> impl Iterator<Item = &'_ mut hecs::World> {
        self.instances.values_mut().map(|instance| &mut instance.world)
    }

    pub(crate) fn define_system(&mut self, name: &str, run: SystemRunCallback) -> Result<()> {
        let uid: UID = name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System '{}' already defined", name));
        }
        self.systems.insert(uid, SystemEntry { 
            name: name.to_string(),
            callbacks: SystemCallbacks { run }
        });
        Ok(())
    }

    pub(crate) fn define_component<C: hecs::Component + Serialize + for<'de> Deserialize<'de>>(&mut self, name: &str) -> Result<()> {
        let uid: UID = name.into();
        let type_id = TypeId::of::<C>();
        if self.components.contains_key(&uid) {
            return Err(anyhow!("Component with name '{}' already defined", name));
        }
        if let Some(uid) = self.component_type_to_uid.get(&type_id) {
            let component = self.components.get(uid).unwrap();
            return Err(anyhow!("Component '{}' defined with the same type id", component.name));
        }
        self.components.insert(uid, ComponentEntry { name: name.to_string(), type_id, component: Box::new(Component::<C> { marker: PhantomData }) });
        self.component_type_to_uid.insert(type_id, uid);
        Ok(())
    }

    pub fn add(&'_ mut self, name: &str) -> Result<UID> {
        let uid = UID::new(name);
        if self.instances.contains_key(&uid) { return Err(anyhow!("ECS already exists")); }
        self.instances.insert(uid, SceneInstance { name: name.to_string(), scheduler: Default::default(), world: Default::default() });
        Ok(uid)
    }

    pub fn remove(&mut self, uid: UID) -> Result<()> {
        if !self.instances.contains_key(&uid) { return Err(anyhow!("ECS not found")); }
        self.instances.remove(&uid);
        Ok(())
    }

    pub fn set_schedule(&mut self, uid: UID, schedule: &SystemScheduleAsset) -> Result<()> {
        let instance = self.instances.get_mut(&uid).with_context(|| "ECS not found")?;
        instance.scheduler.systems = schedule.systems.clone();
        Ok(())
    }
}

pub struct SceneRunner {
    scenes: Vec<UID>,
}

impl SceneRunner {

    pub fn with_scene(mut self, scene: UID) -> Self {
        self.scenes.push(scene);
        self
    }

    pub fn run(self, ctx: &mut ProcessContext) -> Result<()> {
        let mut system_context = SystemContext {
            asset: ctx.asset,
            input: ctx.input,
            signal: ctx.signal,
            script: ctx.script,
            renderer: ctx.renderer,
            delta_time: ctx.delta_time,
            time: ctx.time,
            scene_uid: uid,
        };
        let manager = &mut ctx.scene;
        let instance = manager.instances.get_mut(&uid).with_context(|| "ECS not found")?;
        for system in &instance.scheduler.systems {
            match system {
                SystemScheduleType::Builtin(system_uid) => {
                    let entry = manager.systems.get(system_uid)
                        .with_context(|| format!("Builtin system with UID '{}' from scheduler was not registered", system_uid))?;           
                    (entry.callbacks.run)(&mut system_context, &mut instance.world)
                        .with_context(|| format!("Error raised while executing system '{}'", entry.name))?;
                },
                SystemScheduleType::RhaiScript(_) => {
                    // TODO:
                },
            }
        }
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