use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Model {
    pub mesh: UID,
    pub materials: Vec<UID>,
}

impl Asset for Model {}

impl Model {
    pub const NAME: &'static str = "model";
    pub const UID: UID = UID::new(Model::NAME);
}