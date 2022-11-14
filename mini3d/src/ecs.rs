use std::{any::TypeId, collections::HashMap};

use anyhow::{Result, anyhow, Context};
use hecs::{World, serialize::column::{SerializeContext, DeserializeContext, try_serialize_id}, Archetype, ColumnBatchType, ColumnBatchBuilder};
use serde::{Serialize, Deserialize, ser::SerializeTuple, de::SeqAccess};

use crate::{program::ProgramContext, asset::system_schedule::{SystemScheduleType, SystemSchedule}, uid::UID};

use self::{system::{SystemContext, System, despawn::DespawnEntitiesSystem, free_fly::FreeFlySystem, renderer::{RendererCheckLifecycleSystem, RendererTransferTransformsSystem, RendererUpdateCameraSystem}, rhai::RhaiUpdateScriptsSystem, rotator::RotatorSystem}, component::{camera::CameraComponent, free_fly::FreeFlyComponent, lifecycle::LifecycleComponent, model::ModelComponent, rhai_scripts::RhaiScriptsComponent, rotator::RotatorComponent, script_storage::ScriptStorageComponent, transform::TransformComponent}};

pub mod component;
pub mod system;

struct SystemEntry {
    name: String,
    system: Box<dyn System>,
}

// trait ComponentSerialize {
    // fn try_serialize_id
// }

struct ComponentEntry {
    name: String,
    typeid: TypeId,
}

pub struct ECSManager {
    systems: HashMap<UID, SystemEntry>,
    components: HashMap<UID, ComponentEntry>,
    components_type_to_uid: HashMap<TypeId, UID>,
}

impl Default for ECSManager {
    fn default() -> Self {
        let mut manager = Self { systems: HashMap::default(), components: HashMap::default(), components_type_to_uid: HashMap::default() };

        manager.register_system("despawn_entities", DespawnEntitiesSystem {}).unwrap();
        manager.register_system("free_fly", FreeFlySystem {}).unwrap();
        manager.register_system("renderer_check_lifecycle", RendererCheckLifecycleSystem {}).unwrap();
        manager.register_system("renderer_transfer_transforms", RendererTransferTransformsSystem {}).unwrap();
        manager.register_system("renderer_update_camera", RendererUpdateCameraSystem {}).unwrap();
        manager.register_system("rhai_update_scripts", RhaiUpdateScriptsSystem {}).unwrap();
        manager.register_system("rotator", RotatorSystem {}).unwrap();
        
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

    pub fn register_system<S: System + 'static>(&mut self, name: &str, system: S) -> Result<()> {
        let uid: UID = name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System '{}' already exists", name));
        }
        self.systems.insert(uid, SystemEntry { 
            name: name.to_string(),
            system: Box::new(system) 
        });
        Ok(())
    }

    pub fn register_component<C: 'static>(&mut self, name: &str) -> Result<()> {
        let uid: UID = name.into();
        let typeid = TypeId::of::<C>();
        if self.components.contains_key(&uid) || self.components_type_to_uid.contains_key(&typeid) {
            return Err(anyhow!("Component '{}' already exists", name));
        }
        self.components.insert(uid, ComponentEntry { name: name.to_string(), typeid });
        self.components_type_to_uid.insert(typeid, uid);
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
                        .context(format!("Builtin system with UID '{}' from scheduler was not registered", system_uid))?;           
                    entry.system.run(&mut system_context, world).context(format!("Error raised while executing system '{}'", entry.name))?;
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
}

struct ECSSerializeContext<'a> {
    manager: &'a ECSManager,
}

impl<'a> SerializeContext for ECSSerializeContext<'a> {

    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype. component_types()
            .filter(|&t| self.manager.components_type_to_uid.contains_key(&t))
            .count()
    }

    fn serialize_component_ids<S: SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        out: S,
    ) -> Result<S::Ok, S::Error> {
        out.end()
    }

    fn serialize_components<S: SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        out: S,
    ) -> Result<S::Ok, S::Error> {
        out.end()
    }
}

impl<'a> DeserializeContext for ECSSerializeContext<'a> {

    fn deserialize_component_ids<'de, A>(&mut self, seq: A) -> Result<ColumnBatchType, A::Error>
    where
        A: SeqAccess<'de> {
        todo!()
    }

    fn deserialize_components<'de, A>(
        &mut self,
        entity_count: u32,
        seq: A,
        batch: &mut ColumnBatchBuilder,
    ) -> Result<(), A::Error>
    where
        A: SeqAccess<'de> {
        todo!()
    }
}