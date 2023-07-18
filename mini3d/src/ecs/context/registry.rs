use crate::registry::RegistryManager;

pub struct RegistryContext<'a> {
    pub(crate) manager: &'a RegistryManager,
}

impl<'a> RegistryContext<'a> {}
