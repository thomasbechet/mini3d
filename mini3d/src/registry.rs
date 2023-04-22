use self::{component::ComponentRegistry, system::SystemRegistry, asset::AssetRegistry};

pub mod asset;
pub mod component;
pub mod error;
pub mod system;

#[derive(Default)]
pub(crate) struct RegistryManager {
    pub(crate) assets: AssetRegistry,
    pub(crate) systems: SystemRegistry,
    pub(crate) components: ComponentRegistry,
}