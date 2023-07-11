use self::{component::ComponentRegistry, system::SystemRegistry};

pub mod component;
pub mod error;
pub mod system;

#[derive(Default)]
pub(crate) struct RegistryManager {
    pub(crate) systems: SystemRegistry,
    pub(crate) components: ComponentRegistry,
}
