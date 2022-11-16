use serde::{Serialize, Deserialize};

use super::{Asset, UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTable {
    pub display_name: String,
    pub description: String,
    pub actions: Vec<UID>,
    pub axis: Vec<UID>,
}

impl Asset for InputTable {}