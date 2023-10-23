use mini3d_derive::{Reflect, Serialize};

use crate::{
    feature::core::resource::{Resource, ResourceHookContext},
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};

#[derive(Clone, Serialize, Default)]
pub enum TextureFormat {
    R,
    RG,
    RGB,
    #[default]
    RGBA,
}

#[derive(Clone, Serialize, Default, Reflect)]
pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub(crate) handle: RendererProviderHandle,
}

impl Resource for Texture {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.renderer.on_texture_added(handle, ctx.resource);
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        ctx.renderer.on_texture_removed(handle, ctx.resource);
    }
}
