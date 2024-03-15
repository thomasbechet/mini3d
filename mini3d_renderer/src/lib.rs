#![no_std]

use core::cell::RefCell;

use alloc::boxed::Box;
use mini3d_db::slot_map_key_handle;
use mini3d_utils::slotmap::SlotMap;
use provider::RendererProvider;
use texture::Texture;

pub mod camera;
pub mod canvas;
pub mod color;
pub mod font;
pub mod graphics;
pub mod material;
pub mod mesh;
pub mod provider;
pub mod rasterizer;
pub mod texture;

extern crate alloc;

#[cfg(test)]
extern crate std;

slot_map_key_handle!(TextureHandle);

#[derive(Default)]
pub struct RendererManager {
    provider: RefCell<Box<dyn RendererProvider>>,
    textures: SlotMap<TextureHandle, Texture>,
}

impl RendererManager {
    pub fn set_provider(&self, provider: Box<dyn RendererProvider>) {
        *self.provider.borrow_mut() = provider;
    }
}
