use anyhow::Result;
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{renderer::{backend::RendererBackend, color::Color}, math::rect::IRect};

#[derive(Serialize, Deserialize)]
pub struct Checkbox {
    position: IVec2,
    checked: bool,
}

impl Checkbox {
    
    pub(crate) fn draw(
        &mut self,
        backend: &mut impl RendererBackend,
    ) -> Result<()> {
        backend.canvas_draw_rect(IRect::new(self.position.x, self.position.y, 10, 10), Color::WHITE)?;
        if self.checked {
            backend.canvas_draw_line(self.position, self.position + IVec2::new(10, 10), Color::WHITE)?;
            backend.canvas_draw_line(self.position + IVec2::new(9, 0), self.position + IVec2::new(0, 9), Color::WHITE)?;
        }
        Ok(())
    }
    
    pub fn new(position: IVec2, checked: bool) -> Self {
        Self { position, checked }
    }
}