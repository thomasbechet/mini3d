use serde::{Serialize, Deserialize};

use crate::renderer::backend::SceneCameraHandle;

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
    #[serde(skip)]
    pub(crate) handle: Option<SceneCameraHandle>,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 110.0, handle: None }
    }
}