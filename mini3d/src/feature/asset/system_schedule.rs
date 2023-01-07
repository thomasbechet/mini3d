use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Clone, Serialize, Deserialize)]
pub enum SystemScheduleType {
    Builtin(UID),
    RhaiScript(UID),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SystemScheduleAsset {
    pub systems: Vec<SystemScheduleType>,
}