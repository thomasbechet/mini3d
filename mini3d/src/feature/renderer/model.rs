use mini3d_derive::{Reflect, Resource, Serialize};

use crate::resource::handle::ResourceHandle;

#[derive(Default, Clone, Resource, Serialize, Reflect)]
pub struct Model {
    pub mesh: ResourceHandle,
    pub materials: Vec<ResourceHandle>,
}
