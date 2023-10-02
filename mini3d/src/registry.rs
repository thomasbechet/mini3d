use self::{asset::AssetRegistry, component::ComponentRegistryManager, system::SystemRegistry};

pub mod asset;
pub mod component;
pub mod datatype;
pub mod error;
pub mod system;

#[derive(Default)]
pub struct RegistryManager {
    pub(crate) asset: AssetRegistry,
    pub(crate) component: ComponentRegistryManager,
    pub(crate) system: SystemRegistry,
}

impl RegistryManager {
    pub(crate) fn log(&self) {
        println!("=== COMPONENTS ===");
        for (_, entry) in self.component.entries.iter() {
            println!("- {}", entry.name.as_str());
        }
    }
}
