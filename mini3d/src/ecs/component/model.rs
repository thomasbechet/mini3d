use slotmap::Key;

use crate::{backend::renderer::RendererModelId, asset::model::ModelId};

pub struct ModelComponent {
    pub id: RendererModelId,
    pub model: ModelId,
}

impl From<ModelId> for ModelComponent {
    fn from(model: ModelId) -> Self {
        Self { id: RendererModelId::null(), model }
    }
}