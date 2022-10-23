use slotmap::new_key_type;
use super::Asset;
use crate::asset::{mesh::MeshId, material::MaterialId};

#[derive(Default)]
pub struct Model {
    pub mesh: MeshId,
    pub materials: Vec<MaterialId>,
}

new_key_type! { pub struct ModelId; }

impl Asset for Model {
    type Id = ModelId;
    fn typename() -> &'static str { "model" }
}