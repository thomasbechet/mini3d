use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        component::{Component, ComponentContext, ComponentError},
        entity::Entity,
    },
    renderer::provider::RendererProviderHandle,
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

#[derive(Clone, Serialize, Default, Reflect)]
pub(crate) struct TextureData {
    pub(crate) format: TextureFormat,
    pub(crate) usage: TextureUsage,
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

#[derive(Clone, Serialize, Default, Reflect)]
pub struct Texture {
    data: TextureData,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Texture {
    pub fn new(format: TextureFormat, bytes: Vec<u8>, width: u16, height: u16) -> Self {
        Self {
            data: TextureData {
                format,
                bytes,
                width,
                height,
                ..Default::default()
            },
            handle: RendererProviderHandle::default(),
        }
    }
}

impl Component for Texture {
    fn on_added(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        self.handle = ctx.renderer.add_texture(entity, &self.data)?;
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        ctx.renderer.remove_texture(self.handle)
    }
}
