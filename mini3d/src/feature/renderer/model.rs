use mini3d_derive::{Reflect, Resource, Serialize};

use crate::define_resource_handle;

use super::{material::MaterialHandle, mesh::MeshHandle};

#[derive(Default, Clone, Resource, Serialize, Reflect)]
pub struct Model {
    pub mesh: MeshHandle,
    pub materials: Vec<MaterialHandle>,
}

define_resource_handle!(ModelHandle);
