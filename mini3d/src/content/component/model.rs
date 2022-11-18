use serde::{Serialize, Deserialize};
use slotmap::Key;

use crate::{backend::renderer::RendererModelId, uid::UID};

#[derive(Serialize, Deserialize)]
pub struct ModelComponent {
    pub uid: UID,
    #[serde(skip)]
    pub id: RendererModelId,
}

impl ModelComponent {
    pub fn new(uid: UID) -> Self {
        Self { uid, id: RendererModelId::null() }
    }
}