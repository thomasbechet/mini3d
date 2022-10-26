use serde::{Serialize, Deserialize};
use super::{Asset, mesh::Mesh, material::Material, AssetRef};

#[derive(Default, Serialize, Deserialize)]
pub struct Model {
    pub mesh: AssetRef<Mesh>,
    pub materials: Vec<AssetRef<Material>>,
}

impl Asset for Model {
    fn typename() -> &'static str { "model" }
}