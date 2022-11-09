use serde::{Serialize, Deserialize};

use super::{Asset, UID};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Material {
    pub diffuse: UID,
}

impl Asset for Material {
    fn typename() -> &'static str { "material" }
}