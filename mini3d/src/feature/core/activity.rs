use crate::resource::handle::ResourceHandle;

pub(crate) struct ActivityDescriptor {
    pub(crate) system_sets: Vec<ResourceHandle>,
    pub(crate) prefabs: Vec<ResourceHandle>,
}
