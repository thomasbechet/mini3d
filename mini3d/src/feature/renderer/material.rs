use mini3d_derive::{Component, Reflect, Serialize};

use crate::asset::handle::StaticAsset;

use super::texture::Texture;

#[derive(Default, Clone, Component, Serialize, Reflect)]
pub struct Material {
    pub diffuse: StaticAsset<Texture>,
}
