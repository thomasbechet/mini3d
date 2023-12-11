use crate::{
    feature::renderer::{camera::RenderCameraHandle, texture::TextureHandle},
    math::mat::M4I32F16,
};

pub struct ShadowPassInfo {
    pub camera: RenderCameraHandle,
    pub view_projection: M4I32F16,
    pub target: TextureHandle,
}

pub(crate) struct ShadowPass {}
