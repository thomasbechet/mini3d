use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};

#[derive(Component, Serialize, Reflect, Clone)]
pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: 110.0 }
    }
}

define_resource_handle!(RenderCameraHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct RenderCamera {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl RenderCamera {
    pub const NAME: &'static str = "RTY_RenderCamera";
}

impl Resource for RenderCamera {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}
