use glam::{Mat4, Vec3};

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

impl TransformComponent {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            has_changed: true,
            matrix: Mat4::from_translation(translation),
        }
    }
}