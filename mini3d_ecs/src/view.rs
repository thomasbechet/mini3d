pub mod native_array;
pub mod native_single;

pub use native_array::*;
pub use native_single::*;

use crate::{error::SystemError, instance::SystemResolver};

pub trait SystemView: Default + Clone + 'static {
    fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError>;
}
