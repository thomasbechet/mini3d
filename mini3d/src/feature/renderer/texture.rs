use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
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

#[derive(Clone, Copy, Serialize)]
pub enum TextureWrapMode {
    Clamp,
    Repeat,
    Mirror,
}

define_resource_handle!(TextureHandle);

#[derive(Clone, Serialize, Default, Reflect)]
pub struct Texture {
    pub data: Vec<u8>,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Texture {
    pub const NAME: &'static str = "texture.type";
}

impl Resource for Texture {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        let texture = ctx.resource.get_mut::<Texture>(handle).unwrap();
        ctx.renderer.on_texture_added_hook(texture, handle.into());
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        let texture = ctx.resource.get_mut::<Texture>(handle).unwrap();
        ctx.renderer.on_texture_removed_hook(texture, handle.into());
    }
}
