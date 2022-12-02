use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::backend::ModelHandle};

#[derive(Serialize, Deserialize)]
pub struct ModelComponent {
    pub model: UID,
    #[serde(skip)]
    pub changed: bool,
    #[serde(skip)]
    pub(crate) handle: Option<ModelHandle>,
}

impl ModelComponent {
    pub fn new(model: UID) -> Self {
        Self { model, changed: false, handle: None }
    }
}