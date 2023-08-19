use glam::UVec2;
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{ecs::entity::Entity, renderer::backend::ViewportHandle};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Viewport {
    pub(crate) camera: Option<Entity>,
    pub(crate) resolution: UVec2,
    #[serialize(skip)]
    pub(crate) handle: ViewportHandle,
}

impl Viewport {
    pub fn new(resolution: UVec2, camera: Option<Entity>) -> Self {
        Self {
            camera,
            resolution,
            handle: Default::default(),
        }
    }

    pub fn set_camera(&mut self, camera: Option<Entity>) {
        self.camera = camera;
    }

    pub fn set_resolution(&mut self, resolution: UVec2) {
        self.resolution = resolution;
    }
}
