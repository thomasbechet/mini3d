use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::{uid::UID, ecs::system::SystemCallback};

#[derive(Clone, Copy)]
pub(crate) enum SystemCode {
    Static(SystemCallback),
    Script(UID),
}

pub(crate) struct SystemDefinition {
    pub(crate) name: String,
    pub(crate) code: SystemCode,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: HashMap<UID, SystemDefinition>,
}

impl SystemRegistry {

    fn define(&mut self, definition: SystemDefinition) -> Result<UID> {
        let uid: UID = definition.name.as_str().into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System already defined"));
        }
        self.systems.insert(uid, definition);
        Ok(uid)
    }

    pub(crate) fn define_static(&mut self, name: &str, system: SystemCallback) -> Result<UID> {
        self.define(SystemDefinition { 
            name: name.to_string(),
            code: SystemCode::Static(system),
        })
    }

    pub(crate) fn get(&self, uid: &UID) -> Option<&SystemDefinition> {
        self.systems.get(uid)
    }
}
