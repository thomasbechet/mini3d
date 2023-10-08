use self::{
    component::ComponentRegistryManager, resource::ResourceRegistryManager,
    system::SystemRegistryManager,
};

pub mod component;
pub mod component_type;
pub mod error;
pub mod resource;
pub mod resource_type;

#[derive(Default)]
pub struct RegistryManager {
    pub(crate) resource: ResourceRegistryManager,
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
