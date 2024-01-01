use mini3d_derive::{Component, Reflect, Serialize};

use crate::{ecs::entity::Entity, renderer::provider::RendererProviderHandle};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct StaticMesh {
    pub(crate) model: Entity,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl StaticMesh {
    pub fn new(model: Entity) -> Self {
        Self {
            model,
            handle: Default::default(),
        }
    }
}
