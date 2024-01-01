use mini3d_derive::{fixed, Component, Reflect, Serialize};

use crate::{math::fixed::U32F16, renderer::provider::RendererProviderHandle};

#[derive(Component, Serialize, Reflect, Clone)]
pub struct Camera {
    pub fov: U32F16,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Camera {
    pub fn with_fov(mut self, fov: U32F16) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: fixed!(110),
            handle: Default::default(),
        }
    }
}
