use crate::renderer::{RendererManager, graphics::Graphics, color::Color, RendererStatistics};

pub struct RendererContext<'a> {
    pub(crate) manager: &'a mut RendererManager,
}

impl<'a> RendererContext<'a> {

    pub fn graphics(&mut self) -> &mut Graphics {
        self.manager.graphics()
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.manager.set_clear_color(color)
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.manager.statistics()
    }
}