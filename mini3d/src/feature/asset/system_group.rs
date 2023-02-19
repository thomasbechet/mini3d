use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Clone, Serialize, Deserialize)]
struct ProcedureEntry {
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
    pub const UID: UID = SystemGroup::NAME.into();

    pub fn empty() -> Self {
        Self { procedures: Default::default() }
    }
}