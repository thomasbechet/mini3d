use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, ecs::component::Component};

pub const MAX_RHAI_SCRIPT_COUNT: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RhaiScriptStatus {
    Starting,
    Updating,
    Stopping,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScriptInstance {
    pub uid: UID,
    pub status: RhaiScriptStatus,
}

#[derive(Default, Serialize, Deserialize)]
pub struct RhaiScripts {
    pub instances: [Option<RhaiScriptInstance>; MAX_RHAI_SCRIPT_COUNT],
}

impl Component for RhaiScripts {}

impl RhaiScripts {

    pub const NAME: &'static str = "rhai_scripts";
    pub const UID: UID = UID::new(RhaiScripts::NAME);

    pub fn add(&mut self, uid: UID) -> Result<()> {
        if self.instances.iter().any(|instance| match instance {
            Some(instance) => { instance.uid == uid },
            None => false
        }) {
            return Err(anyhow!("Trying to add existing rhai script"))
        }
        if let Some(instance) = self.instances.iter_mut().find(|instance| instance.is_none()) {
            *instance = Some(RhaiScriptInstance { uid, status: RhaiScriptStatus::Starting });
            Ok(())
        } else {
            Err(anyhow!("No script slot available"))
        }
    }
}