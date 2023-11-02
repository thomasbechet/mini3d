use crate::resource::{handle::ResourceTypeHandle, ResourceManager};

pub mod action;
pub mod axis;
pub mod text;

pub(crate) struct InputResources {
    pub(crate) action: ResourceTypeHandle,
    pub(crate) axis: ResourceTypeHandle,
    pub(crate) text: ResourceTypeHandle,
}

impl InputResources {
    pub(crate) fn define(&mut self, resource: &mut ResourceManager) {}
}
