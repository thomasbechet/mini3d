use glam::Mat4;

use crate::feature::renderer::{camera::RenderCameraHandle, texture::TextureHandle};

pub struct ShadowPassInfo {
    pub camera: RenderCameraHandle,
    pub view_projection: Mat4,
    pub target: TextureHandle,
}

pub(crate) struct ShadowPass {}
