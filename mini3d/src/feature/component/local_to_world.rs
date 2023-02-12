use glam::{Mat4, Vec3, Vec4};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::component::Component};

#[derive(Default, Serialize, Deserialize)]
pub struct LocalToWorld {
    pub matrix: Mat4,
    #[serde(skip)]
    pub(crate) dirty: bool,
}

impl Component for LocalToWorld {}

impl LocalToWorld {

    pub const NAME: &'static str = "local_to_world";
    pub const UID: UID = LocalToWorld::NAME.into();

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