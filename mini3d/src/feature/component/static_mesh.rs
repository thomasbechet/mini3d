use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::backend::SceneModelHandle, ecs::component::Component};

#[derive(Serialize, Deserialize)]
pub struct StaticMesh {
    pub model: UID,
    #[serde(skip)]
    pub changed: bool,
    #[serde(skip)]
    pub(crate) handle: Option<SceneModelHandle>,
}

impl Component for StaticMesh {}

impl StaticMesh {
    pub fn new(model: UID) -> Self {
        Self { model, changed: false, handle: None }
    }
}

impl StaticMesh {
    pub const NAME: &'static str = "static_mesh";
    pub const UID: UID = UID::new(StaticMesh::NAME);
}