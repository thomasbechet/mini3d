use anyhow::Result;
use serde::{Serialize, Deserialize};
use slotmap::Key;

use crate::backend::renderer::{RendererCameraId, RendererBackend};

#[derive(Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov: f32,
    #[serde(skip)]
    pub renderer_id: RendererCameraId,
}

impl CameraComponent {
    pub fn submit(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        self.renderer_id = renderer.add_camera()?;
        Ok(())
    }
    pub fn release(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        renderer.remove_camera(self.renderer_id)?;
        self.renderer_id = RendererCameraId::null();
        Ok(())
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self { fov: 110.0, renderer_id: RendererCameraId::null() }
    }
}