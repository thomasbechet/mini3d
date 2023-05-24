use mini3d_derive::Asset;

use crate::uid::UID;

#[derive(Default, Clone, Asset)]
pub struct Model {
    pub mesh: UID,
    pub materials: Vec<UID>,
}