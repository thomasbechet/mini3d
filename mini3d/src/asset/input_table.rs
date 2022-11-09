use std::collections::HashSet;

use serde::{Serialize, Deserialize};

use super::{Asset, UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTable {
    pub display_name: String,
    pub description: String,
    pub actions: HashSet<UID>,
    pub axis: HashSet<UID>,
}

impl Asset for InputTable {
    fn typename() -> &'static str { "input_table" }
}