use crate::backend::renderer::{RendererCameraId, RendererBackend};

pub struct CameraComponent {
    pub id: RendererCameraId,
    pub fov: f32,
}

impl CameraComponent {

    pub fn new(renderer: &mut dyn RendererBackend) -> Self {
        Self {
            id: renderer.add_camera(),
            fov: 110.0,
        }
    }
}