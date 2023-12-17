use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    renderer::provider::RendererProviderHandle,
    resource::{handle::ResourceHandle, Resource, ResourceHookContext},
};

define_resource_handle!(RenderTransformHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct RenderTransform {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl RenderTransform {
    pub const NAME: &'static str = "RTY_RenderTransform";
}

impl Resource for RenderTransform {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}
