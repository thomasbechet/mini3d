use glam::Mat4;

pub struct TransformComponent {
    pub has_changed: bool,
    pub matrix: Mat4,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self { 
            has_changed: true, 
            matrix: Mat4::IDENTITY 
        }
    }
}