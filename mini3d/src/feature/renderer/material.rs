use mini3d_derive::{Asset, Reflect, Serialize};

use crate::asset::handle::StaticAsset;

use super::texture::Texture;

#[derive(Default, Clone, Asset, Serialize, Reflect)]
pub struct Material {
    pub diffuse: StaticAsset<Texture>,
}
