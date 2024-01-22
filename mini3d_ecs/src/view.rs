pub mod native;

pub use native::*;

use crate::{error::SystemError, instance::SystemResolver};

pub trait SystemView: Default + Clone + 'static {
    fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError>;
}
