use slotmap::Key;

use crate::backend::renderer::CameraHandle;

pub struct CameraComponent {
    pub id: CameraHandle,
    pub fov: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { id: CameraHandle::null(), fov: 110.0 }
    }
}