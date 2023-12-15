use crate::{
    math::mat::M4I32F16,
    renderer::resource::{RenderCameraHandle, TextureHandle},
};

pub struct ShadowPassInfo {
    pub camera: RenderCameraHandle,
    pub view_projection: M4I32F16,
    pub target: TextureHandle,
}

pub(crate) struct ShadowPass {}
