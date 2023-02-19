use glam::{Mat4, Vec3, Quat};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::component::Component};

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Component for Transform {}

impl Transform {

    pub const NAME: &'static str = "transform";
    pub const UID: UID = UID::new(Transform::NAME);

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