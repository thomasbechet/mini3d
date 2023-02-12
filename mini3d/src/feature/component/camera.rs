use serde::{Serialize, Deserialize};

use crate::{renderer::backend::SceneCameraHandle, uid::UID, registry::component::Component};

#[derive(Serialize, Deserialize)]
pub struct Camera {
    pub fov: f32,
    #[serde(skip)]
    pub(crate) handle: Option<SceneCameraHandle>,
}

impl Component for Camera {}

impl Camera {
    pub const NAME: &'static str = "Camera";
    pub const UID: UID = Camera::NAME.into();
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: 110.0, handle: None }
    }
}