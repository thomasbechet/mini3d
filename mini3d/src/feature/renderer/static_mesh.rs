use mini3d_derive::{Component, Reflect, Serialize};

use crate::{renderer::provider::SceneModelHandle, resource::handle::ResourceHandle};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct StaticMesh {
    pub model: ResourceHandle,
    #[serialize(skip)]
    pub(crate) handle: SceneModelHandle,
}

impl StaticMesh {
    pub fn new(model: ResourceHandle) -> Self {
        Self {
            model,
            handle: Default::default(),
        }
    }
}
