use mini3d_derive::{Component, Reflect, Serialize};

use crate::utils::uid::UID;

#[derive(Default, Clone, Component, Serialize, Reflect)]
pub struct Material {
    pub diffuse: UID,
}
