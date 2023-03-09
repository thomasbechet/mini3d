use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPipeline {
    pub(crate) systems: Vec<UID>,
}

impl SystemPipeline {

    pub fn single(system: UID) -> Self {
        Self { systems: vec![system] }
    }

    pub fn new(systems: &[UID]) -> Self {
        Self { systems: systems.to_vec() }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ProcedureEntry {
    pub(crate) name: String,
    pub(crate) priority: i32,
    pub(crate) pipeline: SystemPipeline,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemGroup {
    pub(crate) procedures: HashMap<UID, ProcedureEntry>
}

impl Asset for SystemGroup {}

impl SystemGroup {

    pub const NAME: &'static str = "system_group";
    pub const UID: UID = UID::new(SystemGroup::NAME);

    pub fn empty() -> Self {
        Self { procedures: Default::default() }
    }

    pub fn insert(&mut self, procedure: &str, pipeline: SystemPipeline, priority: i32) {
        let uid = UID::new(procedure);
        self.procedures.insert(uid, ProcedureEntry { name: procedure.to_string(), priority, pipeline });
    }

    pub fn remove(&mut self, procedure: UID) {
        self.procedures.remove(&procedure);
    }
}