use mini3d_derive::{Reflect, Serialize};

use crate::feature::core::resource::{Resource, ResourceTypeHandle};

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Material {
    pub diffuse: ResourceTypeHandle,
}

impl Material {
    pub const NAME: &'static str = "material.type";
}

impl Resource for Material {}
