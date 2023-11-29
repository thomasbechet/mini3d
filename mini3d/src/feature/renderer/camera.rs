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
    handle: GPUCameraHandle,
}

impl Camera {
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 110.0,
            handle: Default::default(),
        }
    }
}

define_resource_handle!(GPUCameraHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct GPUCamera {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl GPUCamera {
    pub const NAME: &'static str = "RTY_GPUCamera";
}

impl Resource for GPUCamera {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}
