use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle, feature::ecs::system::SystemSetHandle, resource::handle::ResourceHandle,
};

use super::resource::Resource;

#[derive(Serialize, Default, Reflect)]
pub struct Activity {
    pub(crate) system_sets: Vec<SystemSetHandle>,
    pub(crate) prefabs: Vec<ResourceHandle>,
    pub(crate) target_fps: u16,
}

impl Activity {
    pub const NAME: &'static str = "RTY_Activity";
}

impl Resource for Activity {}

define_resource_handle!(ActivityHandle);
