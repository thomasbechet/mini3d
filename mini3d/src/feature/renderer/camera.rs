use mini3d_derive::{Component, Reflect, Serialize};

#[derive(Component, Serialize, Reflect, Clone)]
pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: 110.0 }
    }
}
