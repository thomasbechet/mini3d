use anyhow::Result;
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{backend::RendererBackend, RendererResourceManager, color::Color}, asset::AssetManager, math::rect::IRect};

#[derive(Serialize, Deserialize)]
pub struct Sprite {
    texture: UID,
    color: Color,
    position: IVec2,
    extent: IRect,
}

impl Sprite {

    pub(crate) fn draw(
        &mut self, 
        resources: &mut RendererResourceManager,
        backend: &mut impl RendererBackend,
        asset: &AssetManager
    ) -> Result<()> {
        let texture = resources.request_texture(&self.texture, backend, asset)?;
        backend.canvas_blit_rect(texture.handle, self.extent, self.position, self.color, 0)?;
        Ok(())
    }

    pub fn new(texture: UID, position: IVec2, extent: IRect) -> Self {
        Self {
            texture,
            color: Color::WHITE,
            position,
            extent,
        }
    }

    pub fn set_position(&mut self, position: IVec2) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_extent(&mut self, extent: IRect) -> &mut Self {
        self.extent = extent;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
}