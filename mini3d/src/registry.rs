use self::{asset::AssetRegistry, component::ComponentRegistry, system::SystemRegistry};

pub mod asset;
pub mod component;
pub mod error;
pub mod system;

#[derive(Default)]
pub struct RegistryManager {
    pub assets: AssetRegistry,
    pub components: ComponentRegistry,
    pub systems: SystemRegistry,
}

impl RegistryManager {
    pub(crate) fn log(&self) {
        println!("=== COMPONENTS ===");
        for (_, entry) in self.components.entries.iter() {
            println!("- {}", entry.name.as_str());
        }
    }
}
