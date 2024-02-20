use crate::{math::mat::M4I32F16, renderer::component::Camera};

pub enum ShadowPassRenderTarget {
    Texture,
    CubeMap,
}

pub struct ShadowPassRenderInfo<'a> {
    pub camera: &'a Camera,
    pub view_projection: M4I32F16,
    pub target: ShadowPassRenderTarget,
}

pub(crate) struct ShadowPass {}
