use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{color::Color, graphics::Graphics}, math::rect::IRect};

#[derive(Serialize, Deserialize)]
pub struct Sprite {
    texture: UID,
    color: Color,
    position: IVec2,
    extent: IRect,
}

impl Sprite {

    pub fn new(texture: UID, position: IVec2, extent: IRect) -> Self {
        Self {
            texture,
            color: Color::WHITE,
            position,
            extent,
        }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.blit_texture(self.texture, self.extent, self.position, self.color, 0);
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