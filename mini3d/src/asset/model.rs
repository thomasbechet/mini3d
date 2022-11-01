use serde::{Serialize, Deserialize};
use slotmap::new_key_type;
use super::{Asset, AssetUID};

new_key_type! { pub struct ModelId; }

#[derive(Default, Serialize, Deserialize)]
pub struct Model {
    pub mesh: AssetUID,
    pub materials: Vec<AssetUID>,
}

impl Asset for Model {
    type Id = ModelId;
    fn typename() -> &'static str { "model" }
}