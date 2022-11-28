use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
    #[serde(skip)]
    pub handle: UID,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 110.0, handle: UID::null() }
    }
}