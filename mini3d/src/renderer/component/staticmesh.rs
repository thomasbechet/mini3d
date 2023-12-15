use mini3d_derive::{Component, Reflect, Serialize};

use crate::{renderer::provider::RendererProviderHandle, resource::ResourceTypeHandle};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct StaticMesh {
    pub model: ResourceTypeHandle,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl StaticMesh {
    pub fn new(model: ResourceTypeHandle) -> Self {
        Self {
            model,
            handle: Default::default(),
        }
    }
}
