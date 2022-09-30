use glam::{Mat4, Vec3, Vec4, Vec4Swizzles, Quat};

pub struct TransformComponent {
    has_changed: bool,
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
    pub matrix: Mat4,
}

impl TransformComponent {

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            has_changed: true,
            translation,
            rotation: Quat::default(),
            scale: Vec3::ONE,
            matrix: Mat4::from_translation(translation)
        }
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn backward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn down(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Y
    }

    pub fn left(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::NEG_X
    }
}