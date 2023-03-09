use core::cell::RefCell;

use anyhow::Result;

use crate::{registry::{RegistryManager, component::DynamicComponentDefinition}, ecs::{component::Component, system::SystemCallback}, uid::UID};

pub struct RegistryContext<'a> {
    pub(crate) manager: &'a RefCell<RegistryManager>,
}

impl<'a> RegistryContext<'a> {

    pub fn define_static_component<C: Component>(&self, name: &str) -> Result<UID> {
        self.manager.borrow_mut().components.define_static::<C>(name)
    }

    pub fn define_dynamic_component(&self, name: &str, definition: DynamicComponentDefinition) -> Result<UID> {
        self.manager.borrow_mut().components.define_dynamic(name, definition)
    }

    pub fn define_static_system(&self, name: &str, system: SystemCallback) -> Result<()> {
        self.manager.borrow_mut().systems.define_static(name, system)
    }

    pub fn define_rhai_system(&self, name: &str, script: UID) -> Result<()> {
        self.manager.borrow_mut().systems.define_rhai(name, script)
    }
}