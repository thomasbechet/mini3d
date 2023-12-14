use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource, math::vec::V2I32F16};

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
    pub uv0_offset: V2I32F16,
    pub uv0_scale: V2I32F16,
    pub uv1_offset: V2I32F16,
    pub uv1_scale: V2I32F16,
}

impl Material {
    pub const NAME: &'static str = "RTY_Material";
}

impl Resource for Material {}

define_resource_handle!(MaterialHandle);
