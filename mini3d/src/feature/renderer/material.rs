use glam::Vec2;
use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource};

use super::texture::TextureHandle;

#[derive(Default, Reflect, Serialize, Clone)]
pub(crate) enum MaterialType {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Material {
    pub(crate) ty: MaterialType,
    pub tex0: TextureHandle,
    pub tex1: TextureHandle,
    pub uv0_offset: Vec2,
    pub uv0_scale: Vec2,
    pub uv1_offset: Vec2,
    pub uv1_scale: Vec2,
}

impl Material {
    pub const NAME: &'static str = "RTY_Material";
}

impl Resource for Material {}

define_resource_handle!(MaterialHandle);
