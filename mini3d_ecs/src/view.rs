pub mod native;

pub use native::*;

use crate::{container::ContainerEntry, error::SystemError};

pub trait SystemView: Default + Clone + 'static {
    fn resolve(&mut self, container: &ContainerEntry) -> Result<(), SystemError>;
}
