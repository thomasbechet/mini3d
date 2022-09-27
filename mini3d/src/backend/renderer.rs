use glam::Mat4;
use slotmap::new_key_type;

use crate::{graphics::CommandBuffer, asset::{mesh::MeshId, material::MaterialId}};

new_key_type! { 
    pub struct RendererModelId;
    pub struct RendererDynamicMaterialId;
    pub struct RendererDynamicMeshId;
    pub struct RendererDynamicTextureId;
}

pub struct RendererModelDescriptor<'a> {
    pub mesh: MeshId,
    pub materials: &'a [MaterialId],
    pub dynamic_materials: &'a [RendererDynamicMaterialId],
}

pub trait RendererBackend {

    fn add_model(&mut self, descriptor: &RendererModelDescriptor) -> RendererModelId; 
    fn remove_model(&mut self, id: RendererModelId);
    fn transfer_model_transform(&mut self, id: RendererModelId, mat: Mat4);
    
    fn add_dynamic_material(&mut self) -> RendererDynamicMaterialId;
    fn remove_dynamic_material(&mut self, id: RendererDynamicMaterialId);

    fn push_command_buffer(&mut self, command: CommandBuffer);
    fn reset_command_buffers(&mut self);
}

#[derive(Default)]
pub struct DefaultRendererBackend;

impl RendererBackend for DefaultRendererBackend {

    fn add_model(&mut self, _: &RendererModelDescriptor) -> RendererModelId { Default::default() }
    fn remove_model(&mut self, _: RendererModelId) {}
    fn transfer_model_transform(&mut self, _: RendererModelId, _: Mat4) {}

    fn add_dynamic_material(&mut self) -> RendererDynamicMaterialId { Default::default() }
    fn remove_dynamic_material(&mut self, _: RendererDynamicMaterialId) {}

    fn push_command_buffer(&mut self, _: CommandBuffer) {}
    fn reset_command_buffers(&mut self) {}
}