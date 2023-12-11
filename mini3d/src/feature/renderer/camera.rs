use mini3d_derive::{fixed, Component, Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    math::fixed::U32F16,
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};

#[derive(Component, Serialize, Reflect, Clone)]
pub struct Camera {
    pub fov: U32F16,
}

impl Camera {
    pub fn with_fov(mut self, fov: U32F16) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: fixed!(110) }
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
