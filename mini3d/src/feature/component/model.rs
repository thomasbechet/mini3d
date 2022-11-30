use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::RendererHandle};

#[derive(Serialize, Deserialize)]
pub struct ModelComponent {
    pub model: UID,
    #[serde(skip)]
    pub handle: Option<RendererHandle>,
}

impl ModelComponent {
    pub fn new(model: UID) -> Self {
        Self { model, handle: None }
    }
}