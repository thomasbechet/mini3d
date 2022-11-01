use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use crate::asset::{AssetRef, rhai_script::RhaiScript};

pub const MAX_RHAI_SCRIPT_COUNT: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RhaiScriptState {
    Init,
    Update,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScriptInstance {
    pub script: AssetRef<RhaiScript>,
    pub state: RhaiScriptState,
}

#[derive(Serialize, Deserialize)]
pub struct RhaiScriptsComponent {
    pub instances: [Option<RhaiScriptInstance>; MAX_RHAI_SCRIPT_COUNT],
}

impl Default for RhaiScriptsComponent {
    fn default() -> Self {
        Self { instances: Default::default() }
    }
}

impl RhaiScriptsComponent {

    pub fn add(&mut self, script: AssetRef<RhaiScript>) -> Result<()> {
        if self.instances.iter().any(|instance| match instance {
            Some(instance) => { instance.script == script },
            None => false
        }) {
            return Err(anyhow!("Trying to add existing rhai script"))
        }
        if let Some(instance) = self.instances.iter_mut().find(|instance| instance.is_none()) {
            *instance = Some(RhaiScriptInstance { script, state: RhaiScriptState::Init });
            Ok(())
        } else {
            Err(anyhow!("No script slot available"))
        }
    }
}