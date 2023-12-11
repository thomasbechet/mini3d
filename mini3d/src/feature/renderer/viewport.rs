use mini3d_derive::{Component, Reflect, Serialize};

use crate::{ecs::entity::Entity, math::vec::V2U32, renderer::provider::RendererProviderHandle};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Viewport {
    pub(crate) camera: Option<Entity>,
    pub(crate) resolution: V2U32,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Viewport {
    pub fn new(resolution: V2U32, camera: Option<Entity>) -> Self {
        Self {
            camera,
            resolution,
            handle: Default::default(),
        }
    }

    pub fn set_camera(&mut self, camera: Option<Entity>) {
        self.camera = camera;
    }

    pub fn set_resolution(&mut self, resolution: V2U32) {
        self.resolution = resolution;
    }
}
