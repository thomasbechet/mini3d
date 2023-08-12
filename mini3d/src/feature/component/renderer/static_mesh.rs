use mini3d_derive::{Component, Reflect, Serialize};

use crate::{renderer::backend::SceneModelHandle, utils::uid::UID};

#[derive(Default, Component, Serialize, Reflect)]
pub struct StaticMesh {
    pub model: UID,
    #[serialize(skip)]
    pub changed: bool,
    #[serialize(skip)]
    pub(crate) handle: SceneModelHandle,
}

impl StaticMesh {
    pub fn new(model: UID) -> Self {
        Self {
            model,
            changed: false,
            handle: Default::default(),
        }
    }
}
