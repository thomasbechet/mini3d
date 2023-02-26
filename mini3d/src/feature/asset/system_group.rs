use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ProcedureEntry {
    pub(crate) name: String,
    pub(crate) priority: i32,
    pub(crate) systems: Vec<UID>,
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

    pub fn single(procedure: &str, system: UID, priority: i32) -> Self {
        let mut procedures = HashMap::new();
        procedures.insert(procedure.into(), ProcedureEntry {
            name: procedure.into(),
            priority,
            systems: vec![system],
        });
        Self { procedures }
    }
}