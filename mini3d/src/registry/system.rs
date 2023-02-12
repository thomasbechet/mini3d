use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::{uid::UID, context::SystemContext};

pub type SystemCallback = fn(&mut SystemContext) -> Result<()>;

pub(crate) enum SystemKind {
    Compiled(SystemCallback),
    Rhai(UID),
    Lua(UID),
}

pub(crate) struct SystemDefinition {
    pub(crate) name: String,
    pub(crate) kind: SystemKind,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: HashMap<UID, SystemDefinition>,
}

impl SystemRegistry {

    fn define(&mut self, definition: SystemDefinition) -> Result<()> {
        let uid: UID = definition.name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System '{}' already defined", definition.name));
        }
        self.systems.insert(uid, definition);
        Ok(())
    }

    pub(crate) fn define_compiled(&mut self, name: &str, system: SystemCallback) -> Result<()> {
        self.define(SystemDefinition { 
            name: name.to_string(),
            kind: SystemKind::Compiled(system),
        })
    }

    pub(crate) fn define_rhai(&mut self, name: &str, script: UID) -> Result<()> {
        self.define(SystemDefinition { 
            name: name.to_string(),
            kind: SystemKind::Rhai(script),
        })
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&SystemDefinition> {
        self.systems.get(&uid)
    }
}
