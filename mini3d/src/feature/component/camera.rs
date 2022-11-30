use serde::{Serialize, Deserialize};

use crate::renderer::RendererHandle;

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
    #[serde(skip)]
    pub handle: Option<RendererHandle>,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 110.0, handle: None }
    }
}