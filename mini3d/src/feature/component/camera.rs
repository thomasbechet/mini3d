use serde::{Serialize, Deserialize};
use slotmap::Key;

use crate::backend::renderer::RendererCameraId;

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
    #[serde(skip)]
    pub id: RendererCameraId,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 110.0, id: RendererCameraId::null() }
    }
}