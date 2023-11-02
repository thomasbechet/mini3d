use crate::resource::{handle::ResourceTypeHandle, ResourceManager};

pub mod component;
pub mod system;

pub(crate) struct ECSResources {
    pub(crate) component: ResourceTypeHandle,
    pub(crate) system: ResourceTypeHandle,
    pub(crate) system_stage: ResourceTypeHandle,
    pub(crate) system_set: ResourceTypeHandle,
}

impl ECSResources {
    pub(crate) fn define(&mut self, resource: &mut ResourceManager) {}
}
