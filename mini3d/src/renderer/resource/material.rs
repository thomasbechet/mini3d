use mini3d_derive::{Reflect, Serialize};

use crate::{feature::core::resource::ResourceData, resource::handle::ResourceHandle};

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Material {
    pub diffuse: ResourceHandle,
}

impl ResourceData for Material {}
