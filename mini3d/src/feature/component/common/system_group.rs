use mini3d_derive::{Component, Reflect, Serialize};
use std::collections::HashMap;

use crate::uid::UID;

#[derive(Debug, Clone, Serialize)]
pub struct SystemPipeline {
    pub(crate) systems: Vec<UID>,
}

impl SystemPipeline {
    pub fn single(system: UID) -> Self {
        Self {
            systems: vec![system],
        }
    }

    pub fn new(systems: &[UID]) -> Self {
        Self {
            systems: systems.to_vec(),
        }
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct ProcedureEntry {
    pub(crate) name: String,
    pub(crate) priority: i32,
    pub(crate) pipeline: SystemPipeline,
}

#[derive(Clone, Component, Serialize, Reflect, Default)]
pub struct SystemGroup {
    pub(crate) procedures: HashMap<UID, ProcedureEntry>,
}

impl SystemGroup {
    pub fn empty() -> Self {
        Self {
            procedures: Default::default(),
        }
    }

    pub fn insert(&mut self, procedure: &str, pipeline: SystemPipeline, priority: i32) {
        let uid = UID::new(procedure);
        self.procedures.insert(
            uid,
            ProcedureEntry {
                name: procedure.to_string(),
                priority,
                pipeline,
            },
        );
    }

    pub fn remove(&mut self, procedure: UID) {
        self.procedures.remove(&procedure);
    }
}
