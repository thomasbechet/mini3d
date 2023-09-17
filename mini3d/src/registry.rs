use self::{component::ComponentRegistry, system::SystemRegistry};

pub mod component;
pub mod error;
pub mod system;

#[derive(Default)]
pub struct RegistryManager {
    pub systems: SystemRegistry,
    pub components: ComponentRegistry,
}

impl RegistryManager {
    pub(crate) fn log(&self) {
        println!("=== COMPONENTS ===");
        for (_, entry) in self.components.entries.iter() {
            println!("- {}", entry.name.as_str());
        }
    }
}
