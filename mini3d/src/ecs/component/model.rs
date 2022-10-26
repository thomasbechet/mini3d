use slotmap::Key;

use crate::{backend::renderer::ModelHandle, asset::{AssetRef, model::Model}};

pub struct ModelComponent {
    pub handle: ModelHandle,
    pub model: AssetRef<Model>,
}

impl From<AssetRef<Model>> for ModelComponent {
    fn from(model: AssetRef<Model>) -> Self {
        Self { handle: ModelHandle::null(), model }
    }
}