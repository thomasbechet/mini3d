use slotmap::Key;

use crate::backend::renderer::RendererCameraId;

pub struct CameraComponent {
    pub id: RendererCameraId,
    pub fov: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { id: RendererCameraId::null(), fov: 110.0 }
    }
}