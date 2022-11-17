use anyhow::{Result, anyhow, Context};
use std::collections::HashMap;

use crate::{backend::renderer::RendererBackend, asset::AssetManager, input::InputManager, event::AppEvents, script::ScriptManager, ecs::ECSManager, uid::UID};

pub trait ProcessBuilder {
    type BuildData;
    fn build(uid: UID, data: Self::BuildData) -> Self;
}

#[allow(unused_variables)]
pub trait Process {
    fn start(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn update(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn post_update(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
    fn stop(&mut self, ctx: &mut ProcessContext) -> Result<()> { Ok(()) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Starting,
    Running,
    Stopping,
}

pub struct ProcessEntry {
    pub name: String,
    pub uid: UID,
    pub state: ProcessState,
}

pub(crate) struct ProcessManagerContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub ecs: &'a mut ECSManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub events: &'a AppEvents,
    pub delta_time: f64,
}

pub struct ProcessInterface<'a> {
    entries: &'a mut HashMap<UID, ProcessEntry>,
    started_processes: HashMap<UID, Box<dyn Process>>,
}

impl<'a> ProcessInterface<'a> {

    fn new<'b: 'a>(entries: &'b mut HashMap<UID, ProcessEntry>) -> Self {
        Self { entries, started_processes: Default::default() } 
    }

    pub fn start<P>(&mut self, name: &str, data: P::BuildData) -> Result<()> 
        where P: Process + ProcessBuilder + 'static {
        let uid = UID::new(name);
        if self.entries.contains_key(&uid) {
            return Err(anyhow!("Process already exists"));
        }
        self.entries.insert(uid, ProcessEntry { name: name.to_string(), uid, state: ProcessState::Starting });
        self.started_processes.insert(uid, Box::new(P::build(uid, data)));
        Ok(())
    }

    pub fn stop(&mut self, uid: UID) -> Result<()> {
        let entry = self.entries.get_mut(&uid).with_context(|| "Process not found")?;
        entry.state = ProcessState::Stopping;
        Ok(())
    }

    pub fn state(&self, uid: UID) -> Result<ProcessState> {
        let entry = self.entries.get(&uid).with_context(|| "Process not found")?;
        Ok(entry.state)
    }

    pub fn iter(&'a self) -> impl Iterator<Item = &'a ProcessEntry> {
        self.entries.values()
    }
}

pub struct ProcessContext<'a, 'b> {
    pub process: &'a mut ProcessInterface<'b>,
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub ecs: &'a mut ECSManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub events: &'a AppEvents,
    pub delta_time: f64,
}

#[derive(Default)]
pub(crate) struct ProcessManager {
    processes: HashMap<UID, Box<dyn Process>>,
    entries: HashMap<UID, ProcessEntry>,
}

impl ProcessManager {

    pub(crate) fn with_root<P>(data: P::BuildData) -> Self 
        where P: Process + ProcessBuilder + 'static {
        let uid = UID::new("root");
        let mut manager = ProcessManager {
            processes: Default::default(),
            entries: Default::default(),
        };
        manager.processes.insert(uid, Box::new(P::build(uid, data)));
        manager.entries.insert(uid, ProcessEntry { name: "root".to_string(), uid, state: ProcessState::Starting });
        manager
    }

    fn execute_process<F>(&mut self, ctx: &mut ProcessManagerContext, process: UID, mut f: F) -> Result<()> 
        where F: FnMut(&mut dyn Process, &mut ProcessContext) -> Result<()> 
    {
        let process = self.processes.get_mut(&process).unwrap();
        let mut interface = ProcessInterface::new(&mut self.entries);
        let mut ctx = ProcessContext {
            process: &mut interface,
            asset: ctx.asset,
            input: ctx.input,
            script: ctx.script,
            ecs: ctx.ecs,
            renderer: ctx.renderer,
            events: ctx.events,
            delta_time: ctx.delta_time,
        };
        f(process.as_mut(), &mut ctx)?;
        for (uid, process) in interface.started_processes.drain() {
            self.processes.insert(uid, process);
        }
        Ok(())
    }

    pub(crate) fn update(
        &mut self,
        ctx: &mut ProcessManagerContext,
    ) -> Result<()> {

        // Starting processes
        while let Some(uid) = self.entries.iter()
            .find(|(_, entry)| entry.state == ProcessState::Starting)
            .map(|(uid, _)| *uid) {
            self.execute_process(ctx, uid, |process, ctx| {
                process.start(ctx)
            })?;
            let state = &mut self.entries.get_mut(&uid).unwrap().state;
            if *state != ProcessState::Stopping {
                *state = ProcessState::Running;
            }
        }

        // Updating processes
        let running_processes = self.entries.keys().copied().collect::<Vec<_>>();
        for uid in running_processes {
            self.execute_process(ctx, uid, |process, ctx| {
                process.update(ctx)
            })?;
            self.execute_process(ctx, uid, |process, ctx| {
                process.post_update(ctx)
            })?;
        }

        // Stopping processes
        while let Some(uid) = self.entries.iter()
            .find(|(_, entry)| entry.state == ProcessState::Stopping)
            .map(|(uid, _)| *uid) {
            self.execute_process(ctx, uid, |process, ctx| {
                process.stop(ctx)
            })?;
            self.processes.remove(&uid);
            self.entries.remove(&uid);
        }

        Ok(())
    }
}