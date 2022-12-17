use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::backend::SceneModelHandle};

#[derive(Serialize, Deserialize)]
pub struct ModelComponent {
    pub model: UID,
    #[serde(skip)]
    pub changed: bool,
    #[serde(skip)]
    pub(crate) handle: Option<SceneModelHandle>,
}

impl ModelComponent {
    pub fn new(model: UID) -> Self {
        Self { model, changed: false, handle: None }
    }
}