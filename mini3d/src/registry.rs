use self::{component::ComponentRegistryManager, resource::ResourceRegistryManager};

pub mod component;
pub mod error;
pub mod resource;

#[derive(Default)]
pub struct RegistryManager {
    pub(crate) resource: ResourceRegistryManager,
    pub(crate) component: ComponentRegistryManager,
}

impl RegistryManager {
    pub(crate) fn log(&self) {
        println!("=== COMPONENTS ===");
        for (_, entry) in self.component.entries.iter() {
            println!("- {}", entry.name.as_str());
        }
    }
}
