use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Model {
    pub mesh: UID,
    pub materials: Vec<UID>,
}