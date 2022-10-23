use glam::{Mat4, Vec3};
use slotmap::new_key_type;

use crate::{graphics::CommandBuffer, asset::{model::ModelId, AssetManager}};

new_key_type! { 
    pub struct RendererCameraId;
    pub struct RendererModelId;
}

#[derive(Default, Clone, Copy)]
pub struct RendererStatistics {
    pub triangle_count: usize,
    pub draw_count: usize,
    pub viewport: (u32, u32),
}

pub trait RendererBackend {

    fn add_camera(&mut self) -> RendererCameraId;
    fn remove_camera(&mut self, id: RendererCameraId);
    fn update_camera(&mut self, id: RendererCameraId, eye: Vec3, forward: Vec3, up: Vec3, fov: f32);

    fn add_model(&mut self, model: ModelId, asset: &AssetManager) -> RendererModelId; 
    fn remove_model(&mut self, id: RendererModelId);
    fn transfer_model_transform(&mut self, id: RendererModelId, mat: Mat4);
    
    fn push_command_buffer(&mut self, command: CommandBuffer);
    fn reset_command_buffers(&mut self);

    fn statistics(&self) -> RendererStatistics;
}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {

    fn add_camera(&mut self) -> RendererCameraId { Default::default() }
    fn remove_camera(&mut self, _: RendererCameraId) {}
    fn update_camera(&mut self, _: RendererCameraId, _: Vec3, _: Vec3, _: Vec3, _: f32) {}

    fn add_model(&mut self, _: ModelId, _: &AssetManager) -> RendererModelId { Default::default() }
    fn remove_model(&mut self, _: RendererModelId) {}
    fn transfer_model_transform(&mut self, _: RendererModelId, _: Mat4) {}

    fn push_command_buffer(&mut self, _: CommandBuffer) {}
    fn reset_command_buffers(&mut self) {}

    fn statistics(&self) -> RendererStatistics { RendererStatistics { triangle_count: 0, draw_count: 0, viewport: (0, 0) } }
}