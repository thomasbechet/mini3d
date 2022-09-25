use crate::{graphics::ModelId, asset::mesh::MeshId, backend::renderer::RendererBackend};

pub struct ModelComponent {
    pub mesh: MeshId,
    pub id: ModelId,
}

impl ModelComponent {
    pub fn new(renderer: &mut dyn RendererBackend, mesh_id: MeshId) -> Self {
        Self {
            mesh: mesh_id,
            id: renderer.add_model(mesh_id),
        } 
    }
}