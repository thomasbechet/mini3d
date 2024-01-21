pub mod native;

pub use native::*;

use crate::{container::ContainerEntry, entity::Entity, error::SystemError};

pub struct ViewEntityPicker<'a> {
    index: u8,
    types: &'a [Entity],
}

impl<'a> ViewEntityPicker<'a> {
    pub fn pick(&mut self) -> Result<Entity, SystemError> {
        if self.index >= self.types.len() as u8 {
            return Err(SystemError::ConfigError);
        }
        let entity = self.types[self.index as usize];
        self.index += 1;
        Ok(entity)
    }
}

pub trait SystemView: Default + Clone + 'static {
    fn pick_dynamic_view(&mut self, picker: &mut ViewEntityPicker) -> Result<(), SystemError> {
        Ok(())
    }
    fn resolve(&mut self, container: &ContainerEntry) -> Result<(), SystemError>;
}
