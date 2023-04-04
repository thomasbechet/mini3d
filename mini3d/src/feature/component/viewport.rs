use anyhow::Result;
use glam::UVec2;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{renderer::backend::ViewportHandle, ecs::{entity::Entity}, uid::UID, registry::component::{Component, EntityResolver, ComponentDefinition}};

fn default_as_true() -> bool { true }

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub(crate) camera: Option<Entity>,
    pub(crate) resolution: UVec2,
    #[serde(skip)]
    pub(crate) handle: Option<ViewportHandle>,
    #[serde(skip, default = "default_as_true")]
    pub(crate) out_of_date: bool,
}

impl Component for Viewport {}

impl Viewport {

    pub const NAME: &'static str = "viewport";
    pub const UID: UID = UID::new(Viewport::NAME);

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