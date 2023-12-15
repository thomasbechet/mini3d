use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    renderer::provider::RendererProviderHandle,
    resource::{handle::ResourceHandle, Resource, ResourceHookContext},
};

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
