use mini3d_derive::Asset;

use crate::uid::UID;

#[derive(Default, Clone, Asset)]
pub struct Material {
    pub diffuse: UID,
}