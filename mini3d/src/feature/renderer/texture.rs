use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceHookContext},
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};

#[derive(Clone, Serialize, Default)]
pub enum TextureFormat {
    R8,
    R8G8,
    R8G8B8,
    #[default]
    R8G8B8A8,
    R16,
    R16G16,
    R16G16B16,
    R16G16B16A16,
    R32,
    R32G32,
    R32G32B32,
    R32G32B32A32,
}

#[derive(Clone, Copy, Serialize)]
pub enum TextureWrapMode {
    Clamp,
    Repeat,
    Mirror,
}

define_resource_handle!(TextureHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct Texture {
    pub format: TextureFormat,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Texture {
    pub const NAME: &'static str = "RTY_Texture";

    pub fn new(format: TextureFormat, data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            format,
            width,
            height,
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
