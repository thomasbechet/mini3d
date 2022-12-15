use serde::{Serialize, Deserialize};

use crate::renderer::backend::{CameraHandle, ViewportHandle};

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
    #[serde(skip)]
    pub(crate) camera_handle: Option<CameraHandle>,
    #[serde(skip)]
    pub(crate) viewport_handle: Option<ViewportHandle>,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 110.0, camera_handle: None, viewport_handle: None }
    }
}