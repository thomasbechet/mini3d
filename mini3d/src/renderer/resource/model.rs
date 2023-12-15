use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, resource::Resource};

use super::{material::MaterialHandle, mesh::MeshHandle};

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Model {
    pub mesh: MeshHandle,
    pub materials: Vec<MaterialHandle>,
}

impl Model {
    pub const NAME: &'static str = "RTY_Model";
}

impl Resource for Model {}

define_resource_handle!(ModelHandle);
