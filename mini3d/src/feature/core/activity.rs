use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle, feature::ecs::system::SystemSetHandle, resource::handle::ResourceHandle,
};

use super::resource::Resource;

#[derive(Serialize, Default, Reflect)]
pub struct ActivityDescriptor {
    pub(crate) system_sets: Vec<SystemSetHandle>,
    pub(crate) prefabs: Vec<ResourceHandle>,
}

impl ActivityDescriptor {
    pub const NAME: &'static str = "RTY_ActivityDescriptor";
}

impl Resource for ActivityDescriptor {}

define_resource_handle!(ActivityDescriptorHandle);
