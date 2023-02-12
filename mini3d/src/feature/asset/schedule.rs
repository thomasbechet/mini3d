use std::collections::HashMap;

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, scene::{world::World, signal::{SIGNAL_UPDATE, SIGNAL_FIXED_UPDATE, SIGNAL_SCENE_CHANGED}, context::SystemContext}};

#[derive(Serialize, Deserialize)]
struct SystemEntry {
    name: String,
    system: UID,
    active: bool,
}

#[derive(Serialize, Deserialize)]
struct SignalEntry {
    name: String,
    pipeline: Vec<UID>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemGroup {
    name: String,
    systems: Vec<UID>,
}

#[derive(Serialize, Deserialize)]
pub struct Schedule {
    signals: HashMap<UID, SignalEntry>,
    systems: HashMap<UID, SystemEntry>,
    groups: HashMap<UID, SystemGroup>,
}

impl Default for Schedule {
    fn default() -> Self {
        let mut schedule = Self {
            signals: Default::default(),
            systems: Default::default(),
            groups: Default::default(),
        };
        schedule.add_signal(SIGNAL_UPDATE).unwrap();
        schedule.add_signal(SIGNAL_FIXED_UPDATE).unwrap();
        schedule.add_signal(SIGNAL_SCENE_CHANGED).unwrap();
        schedule
    }
}

impl Schedule {

    pub(crate) fn invoke(
        &self, 
        signal: UID,
        registry: &SystemRegistry,
        context: &mut SystemContext,
        world: &mut World,
    ) -> Result<()> {
        if let Some(signal) = self.signals.get(&signal) {
            for system in &signal.pipeline {
                if let Some(entry) = self.systems.get(system) {
                    if entry.active {
                        let definition = registry.get(entry.system)
                            .with_context(|| "System not found")?;
                        match &definition.kind {
                            SystemKind::Compiled(callback) => {
                                (callback)(context, world)?;
                            },
                            SystemKind::Rhai(_) => { 
                                unimplemented!("Rhai not implemented yet")
                            },
                            SystemKind::Lua(_) => { 
                                unimplemented!("Lua not implemented yet")
                            },
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_signal(&mut self, name: &str) -> Result<UID> {
        let uid: UID = name.into();
        if self.signals.contains_key(&uid) {
            return Err(anyhow::anyhow!("Signal with name '{}' already exists", name));
        }
        self.signals.insert(uid, SignalEntry {
            name: name.into(),
            pipeline: Default::default(),
        });
        Ok(uid)
    }

    pub fn add(&mut self, name: &str, system: UID, signal: UID) -> Result<UID> {
        let uid: UID = name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow::anyhow!("System with name '{}' already exists", name));
        }
        self.systems.insert(uid, SystemEntry {
            name: name.into(),
            system,
            active: true,
        });
        self.signals.get_mut(&signal).with_context(|| "Signal not found")?.pipeline.push(uid);
        Ok(uid)
    }
}