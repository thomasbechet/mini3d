#![no_std]

use core::cell::RefCell;

use alloc::boxed::Box;
use camera::{CameraData, CameraHandle};
use mesh::{MeshData, MeshHandle};
use mini3d_utils::slotmap::SlotMap;
use provider::{RendererProvider, RendererProviderError, RendererProviderHandle};
use renderpass::{RenderPassData, RenderPassHandle};
use texture::{TextureData, TextureHandle};
use transform::{RenderTransformData, RenderTransformHandle};

pub mod camera;
pub mod canvas;
pub mod color;
pub mod font;
pub mod graphics;
pub mod material;
pub mod mesh;
pub mod provider;
pub mod rendertarget;
pub mod rasterizer;
pub mod renderpass;
pub mod texture;
pub mod transform;

extern crate alloc;

#[cfg(test)]
extern crate std;

#[derive(Default)]
struct ResourceEntry<T: Default> {
    data: T,
    handle: RendererProviderHandle,
}

impl<T: Default> ResourceEntry<T> {
    fn new(data: T, handle: RendererProviderHandle) -> Self {
        Self { data, handle }
    }
}

#[derive(Default)]
pub struct RendererManager {
    provider: RefCell<Box<dyn RendererProvider>>,
    textures: SlotMap<TextureHandle, ResourceEntry<TextureData>>,
    meshes: SlotMap<MeshHandle, ResourceEntry<MeshData>>,
    transforms: SlotMap<RenderTransformHandle, ResourceEntry<RenderTransformData>>,
    cameras: SlotMap<CameraHandle, ResourceEntry<CameraData>>,
    passes: SlotMap<RenderPassHandle, ResourceEntry<RenderPassData>>,
}

impl RendererManager {
    pub fn set_provider(&self, provider: Box<dyn RendererProvider>) {
        *self.provider.borrow_mut() = provider;
    }

    pub fn create_texture(
        &mut self,
        data: TextureData,
    ) -> Result<TextureHandle, RendererProviderError> {
        let handle = self.provider.get_mut().create_texture(&data)?;
        let handle = self.textures.add(ResourceEntry::new(data, handle));
        Ok(handle)
    }

    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), RendererProviderError> {
        self.provider.get_mut().delete_texture(
            self.textures
                .get(handle)
                .ok_or(RendererProviderError::ResourceNotFound)?
                .handle,
        )?;
        self.textures.remove(handle);
        Ok(())
    }

    pub fn create_mesh(&mut self, data: MeshData) -> Result<MeshHandle, RendererProviderError> {
        let handle = self.provider.get_mut().create_mesh(&data)?;
        let handle = self.meshes.add(ResourceEntry::new(data, handle));
        Ok(handle)
    }

    pub fn delete_mesh(&mut self, handle: MeshHandle) -> Result<(), RendererProviderError> {
        self.provider.get_mut().delete_texture(
            self.meshes
                .get(handle)
                .ok_or(RendererProviderError::ResourceNotFound)?
                .handle,
        )?;
        self.meshes.remove(handle);
        Ok(())
    }
}
