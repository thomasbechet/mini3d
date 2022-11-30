use anyhow::Result;
use glam::{Mat4, Vec3};

use crate::{uid::UID, feature::asset::{texture::Texture, mesh::Mesh, font::Font}};

use super::{command_buffer::CommandBuffer, RendererStatistics};

pub type BackendHandle = UID;

pub struct BackendMaterialDescriptor {
    pub diffuse: BackendHandle,
}

#[allow(unused_variables)]
pub trait RendererBackend {

    /// Resources API
    
    fn mesh_add(&mut self, mesh: &Mesh) -> Result<BackendHandle> { Ok(BackendHandle::null()) }
    fn mesh_remove(&mut self, handle: BackendHandle) -> Result<()> { Ok(()) }

    fn texture_add(&mut self, texture: &Texture) -> Result<BackendHandle> { Ok(BackendHandle::null()) }
    fn texture_remove(&mut self, handle: BackendHandle) -> Result<()> { Ok(()) }

    fn material_add(&mut self, desc: BackendMaterialDescriptor) -> Result<BackendHandle> { Ok(BackendHandle::null()) }
    fn material_remove(&mut self, handle: BackendHandle) -> Result<()> { Ok(()) }

    fn font_add(&mut self, font: &Font) -> Result<BackendHandle> { Ok(BackendHandle::null()) }
    fn font_remove(&mut self, handle: BackendHandle) -> Result<()> { Ok(()) }

    /// Objects API

    fn model_add(&mut self, mesh: BackendHandle) -> Result<BackendHandle> { Ok(BackendHandle::null()) }
    fn model_remove(&mut self, handle: BackendHandle) -> Result<()> { Ok(()) }
    fn model_set_material(&mut self, handle: BackendHandle, material: BackendHandle) -> Result<()> { Ok(()) }
    fn model_transfer_matrix(&mut self, handle: BackendHandle, mat: Mat4) -> Result<()> { Ok(()) }

    fn camera_add(&mut self) -> Result<BackendHandle> { Ok(BackendHandle::null()) }
    fn camera_remove(&mut self, handle: BackendHandle) -> Result<()> { Ok(()) }
    fn camera_update(&mut self, handle: BackendHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> { Ok(()) }
    
    /// Commands API

    fn submit_command_buffer(&mut self, command: &[CommandBuffer]) -> Result<()> { Ok(()) }

    /// Statistics API

    fn retrieve_statistics(&self) -> Result<RendererStatistics> { Ok(RendererStatistics { triangle_count: 0, draw_count: 0, viewport: (0, 0) }) }

}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {}