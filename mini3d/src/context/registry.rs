use core::cell::RefCell;

use crate::registry::RegistryManager;

pub struct RegistryContext<'a> {
    pub(crate) manager: &'a RefCell<RegistryManager>,
}

impl<'a> RegistryContext<'a> {

}