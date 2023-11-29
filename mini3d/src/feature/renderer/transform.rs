use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};

define_resource_handle!(GPUTransformHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct GPUTransform {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl GPUTransform {
    pub const NAME: &'static str = "RTY_GPUTransform";
}

impl Resource for GPUTransform {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}
