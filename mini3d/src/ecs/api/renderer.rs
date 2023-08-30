use crate::renderer::{
    backend::RendererBackend, color::Color, graphics::Graphics, RendererManager, RendererStatistics,
};

pub struct ExclusiveRendererAPI<'a> {
    pub(crate) manager: &'a mut RendererManager,
    pub(crate) backend: &'a mut dyn RendererBackend,
}

impl<'a> ExclusiveRendererAPI<'a> {
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

pub struct ParallelRendererAPI<'a> {
    pub(crate) manager: &'a RendererManager,
}

impl<'a> ParallelRendererAPI<'a> {
    pub fn statistics(&self) -> RendererStatistics {
        self.manager.statistics()
    }
}
