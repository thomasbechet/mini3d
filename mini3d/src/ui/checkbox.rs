use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{renderer::{color::Color, graphics::Graphics}, math::rect::IRect};

#[derive(Serialize, Deserialize)]
pub struct Checkbox {
    position: IVec2,
    checked: bool,
}

impl Checkbox {
    
    pub fn new(position: IVec2, checked: bool) -> Self {
        Self { position, checked }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.draw_rect(IRect::new(self.position.x, self.position.y, 10, 10), Color::WHITE);
        gfx.draw_line(self.position, self.position + IVec2::new(10, 10), Color::WHITE);
        gfx.draw_line(self.position + IVec2::new(9, 0), self.position + IVec2::new(0, 9), Color::WHITE);
    }
}