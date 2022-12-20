use anyhow::Result;
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{backend::{CanvasSpriteHandle, RendererBackend, CanvasHandle}, RendererResourceManager, color::Color, RendererManager}, asset::AssetManager, math::rect::IRect};

fn default_as_true() -> bool { true }

#[derive(Serialize, Deserialize)]
pub struct Sprite {
    texture: UID,
    color: Color,
    z_index: i32,
    position: IVec2,
    extent: IRect,
    #[serde(skip, default = "default_as_true")]
    out_of_date: bool,
    #[serde(skip)]
    handle: Option<CanvasSpriteHandle>,
}

impl Sprite {

    pub(crate) fn update_renderer(
        &mut self,
        canvas: CanvasHandle,
        resources: &mut RendererResourceManager,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
    ) -> Result<()> {
        if self.handle.is_none() {
            let texture_handle = resources.request_texture(&self.texture, backend, asset)?;
            self.handle = Some(backend.canvas_sprite_add(canvas, texture_handle, self.position, self.extent)?);
        }
        if self.out_of_date {
            backend.canvas_sprite_set_position(self.handle.unwrap(), self.position)?;
            backend.canvas_sprite_set_extent(self.handle.unwrap(), self.extent)?;
            backend.canvas_sprite_set_color(self.handle.unwrap(), self.color)?;
            backend.canvas_sprite_set_z_index(self.handle.unwrap(), self.z_index)?;
            self.out_of_date = false;
        }
        Ok(())
    }

    pub(crate) fn release_renderer(
        &mut self,
        backend: &mut dyn RendererBackend,
    ) -> Result<()> {
        if let Some(handle) = self.handle {
            backend.canvas_sprite_remove(handle)?;
        }
        Ok(())
    }

    pub fn new(texture: UID, position: IVec2, extent: IRect) -> Self {
        Self {
            texture,
            color: Color::WHITE,
            z_index: 0,
            position,
            extent,
            out_of_date: true,
            handle: None,
        }
    }

    pub fn set_position(&mut self, position: IVec2) -> &mut Self {
        self.position = position;
        self.out_of_date = true;
        self
    }

    pub fn set_extent(&mut self, extent: IRect) -> &mut Self {
        self.extent = extent;
        self.out_of_date = true;
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self.out_of_date = true;
        self
    }

    pub fn set_z_index(&mut self, z_index: i32) -> &mut Self {
        self.z_index = z_index;
        self.out_of_date = true;
        self
    }
}