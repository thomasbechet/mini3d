use crate::registry::RegistryManager;

pub struct RegistryAPI<'a> {
    pub(crate) manager: &'a RegistryManager,
}

impl<'a> RegistryAPI<'a> {}
