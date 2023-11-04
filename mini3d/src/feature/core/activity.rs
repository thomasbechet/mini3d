use crate::{
    define_resource_handle, feature::ecs::system::SystemSetHandle, resource::handle::ResourceHandle,
};

pub(crate) struct ActivityDescriptor {
    pub(crate) system_sets: Vec<SystemSetHandle>,
    pub(crate) prefabs: Vec<ResourceHandle>,
}

impl ActivityDescriptor {
    pub const NAME: &'static str = "activity_descriptor";
}

define_resource_handle!(ActivityDescriptorHandle);
