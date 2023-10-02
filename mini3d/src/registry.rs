use self::{
    asset::AssetRegistryManager, component::ComponentRegistryManager, system::SystemRegistryManager,
};

pub mod asset;
pub mod component;
pub mod datatype;
pub mod error;
pub mod system;

#[derive(Default)]
pub struct RegistryManager {
    pub(crate) asset: AssetRegistryManager,
    pub(crate) component: ComponentRegistryManager,
    pub(crate) system: SystemRegistryManager,
}

impl RegistryManager {
    pub(crate) fn log(&self) {
        println!("=== COMPONENTS ===");
        for (_, entry) in self.component.entries.iter() {
            println!("- {}", entry.name.as_str());
        }
    }
}
