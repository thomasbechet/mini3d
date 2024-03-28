use mini3d_derive::Serialize;
use mini3d_math::vec::V2I32F16;
use mini3d_utils::slot_map_key;

use crate::texture::TextureHandle;

slot_map_key!(MaterialId);

#[derive(Default, Serialize, Clone)]
pub(crate) enum MaterialType {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Default, Clone, Serialize)]
pub struct Material {
    pub(crate) ty: MaterialType,
    pub(crate) tex0: Option<TextureHandle>,
    pub(crate) tex1: Option<TextureHandle>,
    pub(crate) uv0_offset: V2I32F16,
    pub(crate) uv0_scale: V2I32F16,
    pub(crate) uv1_offset: V2I32F16,
    pub(crate) uv1_scale: V2I32F16,
}

impl Material {}
