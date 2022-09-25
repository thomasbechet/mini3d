use glam::Mat4;
use slotmap::Key;

use crate::{graphics::{ModelId, CommandBuffer}, asset::mesh::MeshId};

pub trait RendererBackend {
    fn add_model(&mut self, mesh_id: MeshId) -> ModelId; 
    fn remove_model(&mut self, id: ModelId);
    fn transfer_model_transform(&mut self, id: ModelId, mat: Mat4);
    fn reset_command_buffers(&mut self);
    fn push_command_buffer(&mut self, command: CommandBuffer);
}

#[derive(Default)]
pub struct DefaultRendererBackend;

impl RendererBackend for DefaultRendererBackend {
    fn add_model(&mut self, _mesh_id: MeshId) -> ModelId { ModelId::null() }
    fn remove_model(&mut self, _id: ModelId) {}
    fn transfer_model_transform(&mut self, _id: ModelId, _mat: Mat4) {}
    fn reset_command_buffers(&mut self) {}
    fn push_command_buffer(&mut self, _command: CommandBuffer) {}
}