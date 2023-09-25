use glam::{Mat4, Vec3, Vec4};
use mini3d_derive::{Component, Reflect, Serialize};

#[derive(Clone, Component, Serialize, Reflect)]
pub struct LocalToWorld {
    pub matrix: Mat4,
    #[serialize(skip)]
    pub(crate) dirty: bool,
}

impl Default for LocalToWorld {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
            dirty: true,
        }
    }
}

impl LocalToWorld {
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
