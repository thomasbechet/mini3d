use core::cell::RefCell;

use anyhow::Result;

use crate::{registry::RegistryManager, uid::UID};

pub struct RegistryContext<'a> {
    pub(crate) manager: &'a RefCell<RegistryManager>,
}

impl<'a> RegistryContext<'a> {

    pub fn define_component(&self, name: &str) -> Result<UID> {
        self.manager.borrow_mut().components.define_dynamic(name)
    }

    pub fn define_script_system(&self, name: &str, script: UID) -> Result<UID> {
        unimplemented!()
    }
}