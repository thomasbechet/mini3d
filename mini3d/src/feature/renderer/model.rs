use mini3d_derive::{Asset, Reflect, Serialize};

use crate::asset::handle::StaticAsset;

use super::{material::Material, mesh::Mesh};

#[derive(Default, Clone, Asset, Serialize, Reflect)]
pub struct Model {
    pub mesh: StaticAsset<Mesh>,
    pub materials: Vec<StaticAsset<Material>>,
}
