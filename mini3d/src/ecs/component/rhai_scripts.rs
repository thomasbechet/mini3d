use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use crate::uid::UID;

use super::Component;

pub const MAX_RHAI_SCRIPT_COUNT: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RhaiScriptState {
    Init,
    Update,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScriptInstance {
    pub uid: UID,
    pub state: RhaiScriptState,
}

#[derive(Default, Serialize, Deserialize)]
pub struct RhaiScriptsComponent {
    pub instances: [Option<RhaiScriptInstance>; MAX_RHAI_SCRIPT_COUNT],
}

impl RhaiScriptsComponent {

    pub fn add(&mut self, uid: UID) -> Result<()> {
        if self.instances.iter().any(|instance| match instance {
            Some(instance) => { instance.uid == uid },
            None => false
        }) {
            return Err(anyhow!("Trying to add existing rhai script"))
        }
        if let Some(instance) = self.instances.iter_mut().find(|instance| instance.is_none()) {
            *instance = Some(RhaiScriptInstance { uid, state: RhaiScriptState::Init });
            Ok(())
        } else {
            Err(anyhow!("No script slot available"))
        }
    }
}

impl Component for RhaiScriptsComponent {}