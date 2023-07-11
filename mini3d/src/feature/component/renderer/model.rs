use mini3d_derive::{Component, Reflect, Serialize};

use crate::uid::UID;

#[derive(Default, Clone, Component, Serialize, Reflect)]
pub struct Model {
    pub mesh: UID,
    pub materials: Vec<UID>,
}
