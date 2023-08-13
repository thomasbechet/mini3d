use mini3d_derive::{Component, Reflect, Serialize};

use crate::asset::handle::StaticAsset;

use super::{material::Material, mesh::Mesh};

#[derive(Default, Clone, Component, Serialize, Reflect)]
pub struct Model {
    pub mesh: StaticAsset<Mesh>,
    pub materials: Vec<StaticAsset<Material>>,
}
