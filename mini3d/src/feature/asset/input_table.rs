use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTable {
    pub display_name: String,
    pub description: String,
    pub actions: Vec<UID>,
    pub axis: Vec<UID>,
}