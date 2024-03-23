use mini3d_derive::{fixed, Serialize};
use mini3d_math::fixed::U32F16;
use mini3d_utils::slot_map_key;

slot_map_key!(CameraId);

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
