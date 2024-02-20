#![no_std]

use core::cell::RefCell;

use alloc::boxed::Box;
use provider::RendererProvider;

pub mod camera;
pub mod canvas;
pub mod color;
pub mod graphics;
pub mod provider;
pub mod rasterizer;

extern crate alloc;

#[cfg(test)]
extern crate std;

#[derive(Default)]
pub struct RendererManager {
    provider: RefCell<Box<dyn RendererProvider>>,
}

impl RendererManager {
    pub fn set_provider(&self, provider: Box<dyn RendererProvider>) {
        *self.provider.borrow_mut() = provider;
    }
}
