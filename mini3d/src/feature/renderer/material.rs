use mini3d_derive::{Reflect, Serialize};

use crate::feature::core::resource::{Resource, ResourceTypeHandle};

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Material {
    pub diffuse: ResourceTypeHandle,
}

impl Resource for Material {}
