use glam::{Mat4, Vec3};
use slotmap::new_key_type;

use crate::{graphics::CommandBuffer, asset::{AssetManager, AssetRef, model::Model}};

new_key_type! { 
    pub struct CameraHandle;
    pub struct ModelHandle;
}

#[derive(Default, Clone, Copy)]
pub struct RendererStatistics {
    pub triangle_count: usize,
    pub draw_count: usize,
    pub viewport: (u32, u32),
}

pub trait RendererBackend {

    fn add_camera(&mut self) -> CameraHandle;
    fn remove_camera(&mut self, handle: CameraHandle);
    fn update_camera(&mut self, handle: CameraHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32);

    fn add_model(&mut self, model: &AssetRef<Model>, asset: &AssetManager) -> ModelHandle; 
    fn remove_model(&mut self, handle: ModelHandle);
    fn transfer_model_transform(&mut self, handle: ModelHandle, mat: Mat4);
    
    fn push_command_buffer(&mut self, command: CommandBuffer);
    fn reset_command_buffers(&mut self);

    fn statistics(&self) -> RendererStatistics;
}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {

    fn add_camera(&mut self) -> CameraHandle { Default::default() }
    fn remove_camera(&mut self, _: CameraHandle) {}
    fn update_camera(&mut self, _: CameraHandle, _: Vec3, _: Vec3, _: Vec3, _: f32) {}

    fn add_model(&mut self, _: &AssetRef<Model>, _: &AssetManager) -> ModelHandle { Default::default() }
    fn remove_model(&mut self, _: ModelHandle) {}
    fn transfer_model_transform(&mut self, _: ModelHandle, _: Mat4) {}

    fn push_command_buffer(&mut self, _: CommandBuffer) {}
    fn reset_command_buffers(&mut self) {}

    fn statistics(&self) -> RendererStatistics { RendererStatistics { triangle_count: 0, draw_count: 0, viewport: (0, 0) } }
}