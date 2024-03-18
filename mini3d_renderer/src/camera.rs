use mini3d_db::slot_map_key_handle;
use mini3d_derive::{fixed, Serialize};
use mini3d_math::fixed::U32F16;

slot_map_key_handle!(CameraHandle);

#[derive(Serialize, Clone)]
pub struct CameraData {
    pub fov: U32F16,
}

impl CameraData {
    pub fn with_fov(mut self, fov: U32F16) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for CameraData {
    fn default() -> Self {
        Self { fov: fixed!(110) }
    }
}
