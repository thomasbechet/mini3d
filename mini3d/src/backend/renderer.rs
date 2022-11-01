use anyhow::Result;
use glam::{Mat4, Vec3};
use slotmap::new_key_type;

use crate::{renderer::CommandBuffer, asset::mesh::SubMesh};

new_key_type! {
    pub struct RendererMeshId;
    pub struct RendererMaterialId;
    pub struct RendererTextureId;
    pub struct RendererCameraId;
    pub struct RendererModelId;
}

pub struct RendererTextureDescriptor<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub struct RendererMaterialDescriptor {
    pub diffuse: RendererTextureId,
}

pub struct RendererMeshDescriptor<'a> {
    pub submeshes: &'a [SubMesh]
}

pub struct RendererModelDescriptor<'a> {
    pub mesh: RendererMeshId,
    pub materials: &'a [RendererMaterialId],
}

#[derive(Default, Clone, Copy)]
pub struct RendererStatistics {
    pub triangle_count: usize,
    pub draw_count: usize,
    pub viewport: (u32, u32),
}

#[allow(unused_variables)]
pub trait RendererBackend {

    fn add_texture(&mut self, desc: &RendererTextureDescriptor) -> Result<RendererTextureId> { Ok(Default::default()) }
    fn remove_texture(&mut self, id: RendererTextureId) -> Result<()> { Ok(()) }

    fn add_material(&mut self, desc: &RendererMaterialDescriptor) -> Result<RendererMaterialId> { Ok(Default::default()) }
    fn remove_material(&mut self, id: RendererMaterialId) -> Result<()> { Ok(()) }

    fn add_mesh(&mut self, desc: &RendererMeshDescriptor) -> Result<RendererMeshId> { Ok(Default::default()) }
    fn remove_mesh(&mut self, id: RendererMeshId) -> Result<()> { Ok(()) }

    fn add_camera(&mut self) -> Result<RendererCameraId> { Ok(Default::default()) }
    fn remove_camera(&mut self, id: RendererCameraId) -> Result<()> { Ok(()) }
    fn update_camera(&mut self, id: RendererCameraId, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> { Ok(()) }

    fn add_model(&mut self, desc: &RendererModelDescriptor) -> Result<RendererModelId> { Ok(Default::default()) }
    fn remove_model(&mut self, id: RendererModelId) -> Result<()> { Ok(()) }
    fn update_model_transform(&mut self, id: RendererModelId, mat: Mat4) -> Result<()> { Ok(()) }
    
    fn push_command_buffer(&mut self, command: CommandBuffer) {}
    fn reset_command_buffers(&mut self) {}

    fn statistics(&self) -> RendererStatistics { RendererStatistics { triangle_count: 0, draw_count: 0, viewport: (0, 0) } }
}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {}