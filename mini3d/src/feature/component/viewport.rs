use glam::UVec2;
use mini3d_derive::Component;

use crate::{renderer::backend::ViewportHandle, ecs::{entity::Entity}};

#[derive(Default, Component)]
#[component(name = "viewport")]
pub struct Viewport {
    pub(crate) camera: Option<Entity>,
    pub(crate) resolution: UVec2,
    #[serialize(skip)]
    pub(crate) handle: Option<ViewportHandle>,
    #[serialize(skip, default = true)]
    pub(crate) out_of_date: bool,
}

impl Viewport {

    pub fn new(resolution: UVec2, camera: Option<Entity>) -> Self {
        Self { camera, resolution, handle: None, out_of_date: true }
    }

    pub fn set_camera(&mut self, camera: Option<Entity>) {
        self.camera = camera;
        self.out_of_date = true;
    }

    pub fn set_resolution(&mut self, resolution: UVec2) {
        self.resolution = resolution;
        self.out_of_date = true;
    }
}