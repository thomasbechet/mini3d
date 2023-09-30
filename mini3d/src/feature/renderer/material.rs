use mini3d_derive::{Asset, Reflect, Serialize};

use crate::asset::handle::AssetHandle;

#[derive(Default, Clone, Asset, Serialize, Reflect)]
pub struct Material {
    pub diffuse: AssetHandle,
}
