use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MaterialAsset {
    pub diffuse: UID,
}