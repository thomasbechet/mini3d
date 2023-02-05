use serde::{Serialize, Deserialize};

use crate::{renderer::backend::SceneCameraHandle, scene::component::Component};

#[derive(Serialize, Deserialize)]
pub struct Camera {
    pub fov: f32,
    #[serde(skip)]
    pub(crate) handle: Option<SceneCameraHandle>,
}

impl Component for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: 110.0, handle: None }
    }
}