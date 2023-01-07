use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ModelAsset {
    pub mesh: UID,
    pub materials: Vec<UID>,
}