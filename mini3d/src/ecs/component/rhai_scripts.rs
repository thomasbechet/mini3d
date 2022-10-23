use anyhow::{Result, anyhow};

use crate::asset::script::RhaiScriptId;

pub const MAX_RHAI_SCRIPT_COUNT: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RhaiScriptState {
    Init,
    Update,
}

#[derive(Clone, Copy)]
pub struct RhaiScriptInstance {
    pub script: RhaiScriptId,
    pub state: RhaiScriptState,
}

pub struct RhaiScriptsComponent {
    pub instances: [Option<RhaiScriptInstance>; MAX_RHAI_SCRIPT_COUNT],
}

impl Default for RhaiScriptsComponent {
    fn default() -> Self {
        Self { instances: [None; MAX_RHAI_SCRIPT_COUNT] }
    }
}

impl RhaiScriptsComponent {

    pub fn add(&mut self, id: RhaiScriptId) -> Result<()> {
        if self.instances.iter().any(|instance| match instance {
            Some(instance) => { instance.script == id },
            None => false
        }) {
            return Err(anyhow!("Trying to add existing rhai script"))
        }
        if let Some(instance) = self.instances.iter_mut().find(|instance| instance.is_none()) {
            *instance = Some(RhaiScriptInstance { script: id, state: RhaiScriptState::Init });
            Ok(())
        } else {
            Err(anyhow!("No script slot available"))
        }
    }
}