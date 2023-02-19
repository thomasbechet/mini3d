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
    pub const NAME: &'static str = "camera";
    pub const UID: UID = UID::new(Camera::NAME);
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: 110.0, handle: None }
    }
}