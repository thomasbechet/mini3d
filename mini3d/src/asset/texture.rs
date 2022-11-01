use anyhow::Result;
use serde::{Serialize, Deserialize};
use slotmap::{new_key_type, Key};
use crate::backend::renderer::{RendererTextureId, RendererBackend, RendererTextureDescriptor};

use super::Asset;

new_key_type! { pub struct TextureId; }

#[derive(Default, Serialize, Deserialize)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn submit(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        self.renderer_id = renderer.add_texture(&RendererTextureDescriptor {
            data: &self.data,
            width: self.width,
            height: self.height,
        })?;
        Ok(())
    }
    pub fn release(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        renderer.remove_texture(self.renderer_id)?;
        self.renderer_id = RendererTextureId::null();
        Ok(())
    }
}

impl Asset for Texture {
    type Id = TextureId;
    fn typename() -> &'static str { "texture" }
}