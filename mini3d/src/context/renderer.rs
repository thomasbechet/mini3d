use core::cell::RefCell;
use std::cell::RefMut;
use crate::renderer::{RendererManager, graphics::Graphics, color::Color, RendererStatistics};

pub struct RendererContext<'a> {
    renderer: RefMut<'a, RendererManager>,
}

impl<'a> RendererContext<'a> {

    pub(crate) fn new(renderer: &'a RefCell<RendererManager>) -> Self {
        Self { renderer: renderer.borrow_mut() }
    }

    pub fn graphics(&mut self) -> &mut Graphics {
        self.renderer.graphics()
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.renderer.set_clear_color(color)
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.renderer.statistics()
    }
}