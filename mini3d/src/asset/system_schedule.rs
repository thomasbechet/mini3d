use serde::{Serialize, Deserialize};

use crate::uid::UID;

use super::Asset;

#[derive(Clone, Serialize, Deserialize)]
pub enum SystemScheduleType {
    Builtin(UID),
    RhaiScript(UID),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SystemSchedule {
    pub systems: Vec<SystemScheduleType>,
}

impl Asset for SystemSchedule {}