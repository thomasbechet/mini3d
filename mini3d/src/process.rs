use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize, Serializer, ser::{SerializeTuple, SerializeSeq}, Deserializer, de::{Visitor, DeserializeSeed}};
use std::{collections::HashMap, any::TypeId, marker::PhantomData};

use crate::{backend::renderer::RendererBackend, asset::AssetManager, input::InputManager, event::AppEvents, script::ScriptManager, ecs::ECSManager, uid::UID, signal::SignalManager};

#[allow(unused_variables)]
pub trait Process {
    fn start(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn pre_update(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn update(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn post_update(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn stop(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
}

pub struct ProcessState {
    pub name: String,
    pub status: ProcessStatus,
    pub type_id: TypeId,
}

struct ProcessType<P: Process + Serialize + for<'de> Deserialize<'de> + 'static> {
    name: String,
    marker: PhantomData<P>,
}

trait AnyProcessType {
    fn name(&self) -> &str;
    fn deserialize(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyProcessInstance>>;
}

impl<P: Process + Serialize + for<'de> Deserialize<'de> + 'static> AnyProcessType for ProcessType<P> {
    fn name(&self) -> &str {
        &self.name
    }
    fn deserialize(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyProcessInstance>> {
        Ok(Box::new(ProcessInstance(P::deserialize(deserializer)?)))
    }
}

pub(crate) struct ProcessManagerContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub ecs: &'a mut ECSManager,
    pub signal: &'a mut SignalManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub events: &'a AppEvents,
    pub delta_time: f64,
    pub time: f64,
}

#[derive(Default)]
struct ProcessMetaTable {
    states: HashMap<UID, ProcessState>,
    types: HashMap<TypeId, Box<dyn AnyProcessType>>,
    type_map: HashMap<UID, TypeId>,
}

impl ProcessMetaTable {

    fn add<P: Process + 'static>(&mut self, name: &str, status: ProcessStatus) -> Result<()> {
        self.add_dynamic(TypeId::of::<P>(), name, status)
    }

    fn add_dynamic(&mut self, type_id: TypeId, name: &str, status: ProcessStatus) -> Result<()> {
        let uid = UID::new(name);
        if self.states.contains_key(&uid) {
            return Err(anyhow!("Process '{}' already exists", name));
        }
        if !self.types.contains_key(&type_id) {
            return Err(anyhow!("Process type not found"));
        }
        self.states.insert(uid, ProcessState { name: name.to_string(), status, type_id });
        Ok(())
    }

    fn remove(&mut self, uid: &UID) {
        self.states.remove(uid);
    }

    fn mark_stopping(&mut self, uid: UID) -> Result<()> {
        let entry = self.states.get_mut(&uid).with_context(|| "Process not found")?;
        entry.status = ProcessStatus::Stopping;
        Ok(())
    }

    fn mark_running(&mut self, uid: UID) -> Result<()> {
        let entry = self.states.get_mut(&uid).with_context(|| "Process not found")?;
        if entry.status != ProcessStatus::Stopping {
            entry.status = ProcessStatus::Running;
        }
        Ok(())
    }

    fn next_stopping(&self) -> Option<UID> {
        self.states.iter()
            .find(|(_, entry)| entry.status == ProcessStatus::Stopping)
            .map(|(uid, _)| *uid)
    }

    fn next_starting(&self) -> Option<UID> {
        self.states.iter()
            .find(|(_, entry)| entry.status == ProcessStatus::Starting)
            .map(|(uid, _)| *uid)
    }

    fn register<P: Process + Serialize + for<'de> Deserialize<'de> + 'static>(&mut self, name: &str) -> Result<()> {
        let uid = UID::new(name);
        if self.type_map.contains_key(&uid) { return Err(anyhow!("Process name already exists")); }
        let type_id = TypeId::of::<P>();
        if self.types.contains_key(&type_id) { return Err(anyhow!("Process type already exists")); }
        self.type_map.insert(uid, type_id);
        self.types.insert(type_id, Box::new(ProcessType::<P> { name: name.to_string(), marker: PhantomData }));
        Ok(())
    }
}

pub struct ProcessInterface<'a> {
    meta: &'a mut ProcessMetaTable,
    started_processes: HashMap<UID, Box<dyn AnyProcessInstance>>,
}

impl<'a> ProcessInterface<'a> {

    fn wrap<'b: 'a>(
        meta: &'b mut ProcessMetaTable,
    ) -> Self {
        Self { meta, started_processes: Default::default() } 
    }
    
    pub fn start<P: Process + Serialize + 'static>(&mut self, name: &str, process: P) -> Result<()> {
        self.meta.add::<P>(name, ProcessStatus::Starting)?;
        self.started_processes.insert(UID::new(name), Box::new(ProcessInstance(process)));
        Ok(())
    }

    pub fn stop(&mut self, uid: UID) -> Result<()> {
        self.meta.mark_stopping(uid)
    }
}

pub struct ProcessContext<'a, 'b> {
    pub process: &'a mut ProcessInterface<'b>,
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub ecs: &'a mut ECSManager,
    pub signal: &'a mut SignalManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub events: &'a AppEvents,
    pub delta_time: f64,
    pub time: f64,
    pub process_uid: UID,
}

struct ProcessInstance<P: Process>(P);

trait AnyProcessInstance {
    fn process(&self) -> &dyn Process;
    fn process_mut(&mut self) -> &mut dyn Process;
    fn type_id(&self) -> TypeId;
    fn as_serialize(&self) -> &dyn erased_serde::Serialize;
}

impl<P: Process + Serialize + 'static> AnyProcessInstance for ProcessInstance<P> {
    fn process(&self) -> &dyn Process { &self.0 }
    fn process_mut(&mut self) -> &mut dyn Process { &mut self.0 }
    fn type_id(&self) -> TypeId { TypeId::of::<P>() }
    fn as_serialize(&self) -> &dyn erased_serde::Serialize { &self.0 }
}

#[derive(Default)]
pub struct ProcessManager {
    instances: HashMap<UID, Box<dyn AnyProcessInstance>>,
    meta: ProcessMetaTable,
}

impl ProcessManager {

    fn execute_process<F>(&mut self, ctx: &mut ProcessManagerContext, uid: UID, mut f: F) -> Result<()> 
        where F: FnMut(&mut dyn Process, &mut ProcessContext) -> Result<()> 
    {
        let instance = self.instances.get_mut(&uid).unwrap();
        let mut interface = ProcessInterface::wrap(&mut self.meta);
        let mut ctx = ProcessContext {
            process: &mut interface,
            asset: ctx.asset,
            input: ctx.input,
            script: ctx.script,
            ecs: ctx.ecs,
            signal: ctx.signal,
            renderer: ctx.renderer,
            events: ctx.events,
            delta_time: ctx.delta_time,
            time: ctx.time,
            process_uid: uid,
        };
        f(instance.process_mut(), &mut ctx)?;
        for (uid, process) in interface.started_processes.drain() {
            self.instances.insert(uid, process);
        }
        Ok(())
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut ProcessManagerContext,
    ) -> Result<()> {

        // Starting processes
        while let Some(uid) = self.meta.next_starting() {
            self.execute_process(ctx, uid, |process, ctx| {
                process.start(ctx)
            })?;
            self.meta.mark_running(uid)?;
        }

        // Updating processes
        let running_processes = self.meta.states.keys().copied().collect::<Vec<_>>();
        for uid in &running_processes {
            self.execute_process(ctx, *uid, |process, ctx| {
                process.pre_update(ctx)
            })?;
        }
        for uid in &running_processes {
            self.execute_process(ctx, *uid, |process, ctx| {
                process.update(ctx)
            })?;
        }
        for uid in &running_processes {
            self.execute_process(ctx, *uid, |process, ctx| {
                process.post_update(ctx)
            })?;
        }
        
        // Stopping processes
        while let Some(uid) = self.meta.next_stopping() {
            self.execute_process(ctx, uid, |process, ctx| {
                process.stop(ctx)
            })?;
            self.instances.remove(&uid);
            self.meta.remove(&uid);
        }

        Ok(())
    }

    pub(crate) fn serialize_process<S: Serializer>(&self, process: UID, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;
        let instance = self.instances.get(&process)
            .with_context(|| "Process not found").map_err(Error::custom)?;
        let meta = self.meta.states.get(&process)
            .with_context(|| "Process state not found").map_err(Error::custom)?;
        let process_type = self.meta.types.get(&instance.type_id())
            .with_context(|| "Process type not found").map_err(Error::custom)?;
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(process_type.name())?;
        tuple.serialize_element(&meta.name)?;
        tuple.serialize_element(&meta.status)?;
        tuple.serialize_element(&instance.as_serialize())?;
        tuple.end()
    }

    pub(crate) fn deserialize_process<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct ProcessInstanceVisitor<'a> {
            instances: &'a mut HashMap<UID, Box<dyn AnyProcessInstance>>,
            meta: &'a mut ProcessMetaTable,
        }
        impl<'de, 'a> Visitor<'de> for ProcessInstanceVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Process instance")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: serde::de::SeqAccess<'de>
            {
                use serde::de::Error;
                struct ProcessDeserialize<'a> {
                    process_type: &'a dyn AnyProcessType, 
                }
                impl<'de> DeserializeSeed<'de> for ProcessDeserialize<'_> {
                    type Value = Box<dyn AnyProcessInstance>;
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de>
                    {
                        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
                        self.process_type.deserialize(&mut deserializer)
                            .map_err(Error::custom)
                    }
                }
                // Read meta fields
                let type_name: String = seq.next_element()?.with_context(|| "Expect process type").map_err(Error::custom)?;
                let name: String = seq.next_element()?.with_context(|| "Expect name").map_err(Error::custom)?;
                let status: ProcessStatus = seq.next_element()?.with_context(|| "Expect status").map_err(Error::custom)?;
                let type_uid = UID::new(&type_name);
                // Find process type
                let type_id = self.meta.type_map.get(&type_uid)
                    .with_context(|| format!("Process type '{}' not found", type_name)).map_err(Error::custom)?;
                let process_type = self.meta.types.get(type_id)
                    .with_context(|| "Process type not found").map_err(Error::custom)?;
                // Deserialize the process
                let process = seq.next_element_seed(ProcessDeserialize { process_type: process_type.as_ref() })?
                    .with_context(|| "Expect process data").map_err(Error::custom)?;
                // Try insert the process
                self.meta.add_dynamic(*type_id, &name, status).map_err(Error::custom)?;
                self.instances.insert(UID::new(&name), process);
                Ok(())
            }
        }
        deserializer.deserialize_tuple(4, ProcessInstanceVisitor { instances: &mut self.instances, meta: &mut self.meta })?;
        Ok(())
    }

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct ProcessSerialize<'a> {
            manager: &'a ProcessManager,
            uid: UID,
        }
        impl<'a> Serialize for ProcessSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                self.manager.serialize_process(self.uid, serializer)
            }
        }
        let mut seq = serializer.serialize_seq(Some(self.instances.len()))?;
        for uid in self.instances.keys() {
            seq.serialize_element(&ProcessSerialize { manager: self, uid: *uid })?;
        }
        seq.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct ProcessVisitor<'a> {
            manager: &'a mut ProcessManager,
        }
        impl<'de, 'a> Visitor<'de> for ProcessVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Process")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: serde::de::SeqAccess<'de> {
                struct ProcessDeserializeSeed<'a> {
                    manager: &'a mut ProcessManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ProcessDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        self.manager.deserialize_process(deserializer)
                    }
                }
                while seq.next_element_seed(ProcessDeserializeSeed { manager: self.manager })?.is_some() {}
                Ok(())
            }
        }
        self.instances.clear();
        self.meta.states.clear();
        deserializer.deserialize_seq(ProcessVisitor { manager: self })?;
        Ok(())
    }

    pub fn register<P: Process + Serialize + for<'de> Deserialize<'de> + 'static>(&mut self, name: &str) -> Result<()> {
        self.meta.register::<P>(name)?;
        Ok(())
    }

    pub fn start<P: Process + Serialize + 'static>(&mut self, name: &str, process: P) -> Result<()> {
        self.meta.add::<P>(name, ProcessStatus::Starting)?;
        self.instances.insert(UID::new(name), Box::new(ProcessInstance(process)));
        Ok(())
    }
}