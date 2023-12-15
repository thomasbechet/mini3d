#[derive(Component, Serialize, Reflect, Clone)]
pub struct Camera {
    pub fov: U32F16,
}

impl Camera {
    pub fn with_fov(mut self, fov: U32F16) -> Self {
        self.fov = fov;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self { fov: fixed!(110) }
    }
}
