use crate::renderer::{
    backend::RendererBackend, color::Color, graphics::Graphics, RendererManager, RendererStatistics,
};

pub struct ExclusiveRendererContext<'a> {
    pub(crate) manager: &'a mut RendererManager,
    pub(crate) backend: &'a dyn RendererBackend,
}

impl<'a> ExclusiveRendererContext<'a> {
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

pub struct ParallelRendererContext<'a> {
    pub(crate) manager: &'a RendererManager,
}

impl<'a> ParallelRendererContext<'a> {
    pub fn statistics(&self) -> RendererStatistics {
        self.manager.statistics()
    }
}
