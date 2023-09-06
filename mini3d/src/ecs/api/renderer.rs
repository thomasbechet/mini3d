use crate::renderer::{
    color::Color, graphics::Graphics, server::RendererServer, RendererManager, RendererStatistics,
};

pub struct ExclusiveRendererAPI<'a> {
    pub(crate) manager: &'a mut RendererManager,
    pub(crate) server: &'a mut dyn RendererServer,
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
