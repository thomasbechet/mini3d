use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    renderer::provider::RendererProviderHandle,
    resource::{handle::ResourceHandle, Resource, ResourceHookContext},
};

#[derive(Clone, Serialize, Default)]
pub enum TextureFormat {
    #[default]
    Color,
    ColorAlpha,
    Depth,
    DepthStencil,
    CubeMap,
}

#[derive(Clone, Copy, Serialize)]
pub enum TextureWrapMode {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Default, Clone, Copy, Serialize, Reflect)]
pub enum TextureUsage {
    #[default]
    Sample,
    RenderTarget,
    Present,
}

define_resource_handle!(TextureHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct Texture {
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Texture {
    pub const NAME: &'static str = "RTY_Texture";

    pub fn new(format: TextureFormat, data: Vec<u8>, width: u16, height: u16) -> Self {
        Self {
            data,
            format,
            width,
            height,
            usage: TextureUsage::Sample,
            handle: RendererProviderHandle::null(),
        }
    }
}

impl Resource for Texture {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        let texture = ctx.resource.native_mut::<Texture>(handle).unwrap();
        ctx.renderer.on_texture_added_hook(texture, handle.into());
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        let texture = ctx.resource.native_mut::<Texture>(handle).unwrap();
        ctx.renderer.on_texture_removed_hook(texture, handle.into());
    }
}
