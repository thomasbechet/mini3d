use mini3d_derive::Component;

use crate::{uid::UID, renderer::backend::SceneModelHandle};

#[derive(Default, Component)]
pub struct StaticMesh {
    pub model: UID,
    #[serialize(skip)]
    pub changed: bool,
    #[serialize(skip)]
    pub(crate) handle: Option<SceneModelHandle>,
}

impl StaticMesh {
    pub fn new(model: UID) -> Self {
        Self { model, changed: false, handle: None }
    }
}