use core::cell::RefCell;

use crate::{
    registry::{error::RegistryError, RegistryManager},
    utils::uid::UID,
};

pub struct RegistryContext<'a> {
    pub(crate) manager: &'a RefCell<RegistryManager>,
}

impl<'a> RegistryContext<'a> {
    pub fn define_component(&self, name: &str) -> Result<UID, RegistryError> {
        self.manager.borrow_mut().components.define_dynamic(name)
    }

    pub fn define_script_system(&self, _name: &str, _script: UID) -> Result<UID, RegistryError> {
        unimplemented!()
    }
}
