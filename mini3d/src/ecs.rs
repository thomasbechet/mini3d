use std::{any::{TypeId, type_name}, collections::HashMap, marker::PhantomData};

use anyhow::{Result, anyhow, Context};
use hecs::{World, serialize::column::{SerializeContext, DeserializeContext}, Archetype, ColumnBatchType, ColumnBatchBuilder, ArchetypeColumn};
use serde::{Serialize, Deserialize, ser::SerializeTuple, de::{SeqAccess, DeserializeSeed, Visitor}, Serializer, Deserializer};

use crate::{program::ProgramContext, asset::{system_schedule::{SystemScheduleType, SystemSchedule}, AssetManager}, uid::UID, input::InputManager, script::ScriptManager, backend::renderer::RendererBackend};

use self::{system::{despawn, free_fly, renderer, rhai, rotator}, component::{camera::CameraComponent, free_fly::FreeFlyComponent, lifecycle::LifecycleComponent, model::ModelComponent, rhai_scripts::RhaiScriptsComponent, rotator::RotatorComponent, script_storage::ScriptStorageComponent, transform::TransformComponent}};

pub mod component;
pub mod system;

pub struct SystemContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub delta_time: f64,
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

pub struct ECSManager {
    systems: HashMap<UID, SystemEntry>,
    components: HashMap<UID, ComponentEntry>,
    component_type_to_uid: HashMap<TypeId, UID>,
}

impl Default for ECSManager {
    fn default() -> Self {
        let mut manager = Self { systems: HashMap::default(), components: HashMap::default(), component_type_to_uid: HashMap::default() };

        manager.register_system("despawn_entities", despawn::run).unwrap();
        manager.register_system("free_fly", free_fly::run).unwrap();
        manager.register_system("renderer_check_lifecycle", renderer::check_lifecycle).unwrap();
        manager.register_system("renderer_transfer_transforms", renderer::transfer_transforms).unwrap();
        manager.register_system("renderer_update_camera", renderer::update_camera).unwrap();
        manager.register_system("rhai_update_scripts", rhai::update_scripts).unwrap();
        manager.register_system("rotator", rotator::run).unwrap();
        
        manager.register_component::<CameraComponent>("camera").unwrap();
        manager.register_component::<FreeFlyComponent>("free_fly").unwrap();
        manager.register_component::<LifecycleComponent>("lifecycle").unwrap();
        manager.register_component::<ModelComponent>("model").unwrap();
        manager.register_component::<RhaiScriptsComponent>("rhai_scripts").unwrap();
        manager.register_component::<RotatorComponent>("rotator").unwrap();
        manager.register_component::<ScriptStorageComponent>("script_storage").unwrap();
        manager.register_component::<TransformComponent>("transform").unwrap();
        
        manager
    }
}

impl ECSManager {

    pub fn register_system(&mut self, name: &str, run: SystemRunCallback) -> Result<()> {
        let uid: UID = name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System '{}' already exists", name));
        }
        self.systems.insert(uid, SystemEntry { 
            name: name.to_string(),
            callbacks: SystemCallbacks { run }
        });
        Ok(())
    }

    pub fn register_component<C: hecs::Component + Serialize + for<'de> Deserialize<'de>>(&mut self, name: &str) -> Result<()> {
        let uid: UID = name.into();
        let type_id = TypeId::of::<C>();
        if self.components.contains_key(&uid) {
            return Err(anyhow!("Component with name '{}' already registered", name));
        }
        if let Some(uid) = self.component_type_to_uid.get(&type_id) {
            let component = self.components.get(uid).unwrap();
            return Err(anyhow!("Component '{}' registered with the same type id", component.name));
        }
        self.components.insert(uid, ComponentEntry { name: name.to_string(), type_id, component: Box::new(Component::<C> { marker: PhantomData }) });
        self.component_type_to_uid.insert(type_id, uid);
        Ok(())
    }

}

#[derive(Default, Serialize, Deserialize)]
pub struct SystemScheduler {
    systems: Vec<SystemScheduleType>,
}

impl SystemScheduler {
    pub(crate) fn run(&self, ctx: &mut ProgramContext, world: &mut World) -> Result<()> {
        let mut system_context = SystemContext {
            asset: ctx.asset,
            input: ctx.input,
            script: ctx.script,
            renderer: ctx.renderer,
            delta_time: ctx.delta_time,
        };
        for system in &self.systems {
            match system {
                SystemScheduleType::Builtin(system_uid) => {
                    let entry = ctx.ecs.systems.get(system_uid)
                        .with_context(|| format!("Builtin system with UID '{}' from scheduler was not registered", system_uid))?;           
                    (entry.callbacks.run)(&mut system_context, world)
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

#[derive(Default)]
pub struct ECS {
    pub world: World,
    scheduler: SystemScheduler,
}

impl ECS {

    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn set_schedule(&mut self, schedule: &SystemSchedule) -> Result<()> {
        self.scheduler.systems = schedule.systems.clone();
        Ok(())
    }

    pub fn progress(&mut self, ctx: &mut ProgramContext) -> Result<()> {
        self.scheduler.run(ctx, &mut self.world)?;
        Ok(())
    }

    pub fn serialize<S: Serializer>(&self, manager: &ECSManager, serializer: S) -> Result<S::Ok, S::Error> {
        hecs::serialize::column::serialize(&self.world, &mut ECSSerializeContext { manager, components: Default::default() }, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(&mut self, manager: &ECSManager, deserializer: D) -> Result<()> {
        self.world = hecs::serialize::column::deserialize(&mut ECSDeserializeContext { manager, components: Default::default() }, deserializer)
            .map_err(|err| anyhow!("Failed to deserialize ECS: {}", err.to_string()))?;
        Ok(())
    }
}

struct ECSSerializeContext<'a> {
    manager: &'a ECSManager,
    components: Vec<UID>,
}

impl<'a> SerializeContext for ECSSerializeContext<'a> {

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

struct ECSDeserializeContext<'a> {
    manager: &'a ECSManager,
    components: Vec<UID>,
}

impl<'a> DeserializeContext for ECSDeserializeContext<'a> {

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