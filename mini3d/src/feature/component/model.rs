use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::backend::SceneModelHandle, scene::component::Component};

#[derive(Serialize, Deserialize)]
pub struct Model {
    pub model: UID,
    #[serde(skip)]
    pub changed: bool,
    #[serde(skip)]
    pub(crate) handle: Option<SceneModelHandle>,
}

impl Component for Model {}

impl Model {
    pub fn new(model: UID) -> Self {
        Self { model, changed: false, handle: None }
    }
}