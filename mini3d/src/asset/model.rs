use serde::{Serialize, Deserialize};
use super::{Asset, UID};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Model {
    pub mesh: UID,
    pub materials: Vec<UID>,
}

impl Asset for Model {}