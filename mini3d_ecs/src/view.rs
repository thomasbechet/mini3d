pub mod native;

pub use native::*;

pub trait SystemView: Default + Clone {}
