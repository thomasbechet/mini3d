use mini3d_derive::{Component, Reflect, Serialize};

use crate::math::{
    mat::{M4, M4I32F16},
    vec::{V3I32F16, V4},
};

#[derive(Clone, Component, Serialize, Reflect)]
pub struct LocalToWorld {
    pub matrix: M4I32F16,
    #[serialize(skip)]
    pub(crate) dirty: bool,
}

impl Default for LocalToWorld {
    fn default() -> Self {
        Self {
            matrix: M4::IDENTITY,
            dirty: true,
        }
    }
}

impl LocalToWorld {
    pub fn translation(&self) -> V3I32F16 {
        self.matrix.waxis.xyz()
    }

    pub fn forward(&self) -> V3I32F16 {
        (self.matrix * V4::Z).xyz()
    }

    pub fn up(&self) -> V3I32F16 {
        (self.matrix * V4::Y).xyz()
    }
}
