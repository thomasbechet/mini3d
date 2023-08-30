use crate::registry::RegistryManager;

pub struct ParallelRegistryAPI<'a> {
    pub(crate) manager: &'a RegistryManager,
}

impl<'a> ParallelRegistryAPI<'a> {}

pub struct ExclusiveRegistryAPI<'a> {
    pub(crate) manager: &'a mut RegistryManager,
}

impl<'a> ExclusiveRegistryAPI<'a> {}
