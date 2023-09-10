use self::{component::ComponentRegistry, system::SystemRegistry};

pub mod component;
pub mod error;
pub mod system;

#[derive(Default)]
pub struct RegistryManager {
    pub(crate) systems: SystemRegistry,
    pub(crate) components: ComponentRegistry,
}

impl RegistryManager {
    pub(crate) fn log(&self) {
        println!("=== SYSTEMS ===");
        for (_, entry) in self.systems.systems.iter() {
            println!("- {}", entry.name.as_str());
        }
        println!("=== STAGES ===");
        for (_, entry) in self.systems.stages.iter() {
            println!("- {}", entry.name.as_str());
        }
        println!("=== COMPONENTS ===");
        for (_, entry) in self.components.entries.iter() {
            println!("- {}", entry.name.as_str());
        }
    }
}
