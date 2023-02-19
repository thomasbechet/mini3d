use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Material {
    pub diffuse: UID,
}

impl Asset for Material {}

impl Material {
    pub const NAME: &'static str = "material";
    pub const UID: UID = Material::NAME.into();
}