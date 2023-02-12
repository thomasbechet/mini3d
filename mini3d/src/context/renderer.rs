use core::cell::RefCell;
use crate::renderer::{RendererManager, graphics::Graphics, color::Color, RendererStatistics};

pub struct RendererContext<'a> {
    renderer: &'a RefCell<RendererManager>,
}

impl<'a> RendererContext<'a> {

    pub(crate) fn new(renderer: &'a RefCell<RendererManager>) -> Self {
        Self { renderer }
    }

    pub fn graphics(&self) -> &mut Graphics {
        self.renderer.borrow_mut().graphics()
    }

    pub fn set_clear_color(&self, color: Color) {
        self.renderer.borrow().set_clear_color(color)
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.renderer.borrow().statistics()
    }
}