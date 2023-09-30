use mini3d_derive::{Component, Reflect, Serialize};

use crate::{asset::handle::AssetHandle, renderer::provider::SceneModelHandle};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct StaticMesh {
    pub model: AssetHandle,
    #[serialize(skip)]
    pub(crate) handle: SceneModelHandle,
}

impl StaticMesh {
    pub fn new(model: AssetHandle) -> Self {
        Self {
            model,
            handle: Default::default(),
        }
    }
}
