use glam::{Mat4, Vec3, Quat, Vec4};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TransformComponent {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl TransformComponent {

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::default(),
            scale: Vec3::ONE,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
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

#[derive(Default, Serialize, Deserialize)]
pub struct LocalToWorldComponent {
    pub matrix: Mat4,
    #[serde(skip)]
    pub(crate) dirty: bool,
}

impl LocalToWorldComponent {

    pub fn translation(&self) -> Vec3 {
        self.matrix.w_axis.truncate()
    }

    pub fn forward(&self) -> Vec3 {
        (self.matrix * Vec4::Z).truncate()
    }

    pub fn up(&self) -> Vec3 {
        (self.matrix * Vec4::Y).truncate()
    }
}