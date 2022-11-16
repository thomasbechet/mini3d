use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Material {
    pub diffuse: UID,
}