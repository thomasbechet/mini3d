use crate::{backend::renderer::{RendererBackend, RendererModelId, RendererModelDescriptor}};

pub struct ModelComponent {
    pub id: RendererModelId,
}

impl ModelComponent {
    pub fn new(renderer: &mut dyn RendererBackend, descriptor: &RendererModelDescriptor) -> Self {
        Self { id: renderer.add_model(descriptor) } 
    }
}