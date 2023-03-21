use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{color::Color, graphics::Graphics}, math::rect::IRect, ui::event::{Event, EventContext}};

use super::Widget;

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

impl Widget for Sprite {

    fn handle_event(&mut self, _ctx: &mut EventContext, _event: &Event) -> bool { false }

    fn render(&self, gfx: &mut Graphics, offset: IVec2, _time: f64) {
        gfx.blit_texture(self.texture, self.extent, self.position + offset, self.color, 0);
    }

    fn extent(&self) -> IRect {
        self.extent.translate(self.position)
    }

    fn is_focusable(&self) -> bool { false }

    fn is_selectable(&self) -> bool { false }
}