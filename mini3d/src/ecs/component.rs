use serde::{Deserialize, Serialize};

pub mod camera;
pub mod free_fly;
pub mod rhai_scripts;
pub mod lifecycle;
pub mod model;
pub mod rotator;
pub mod script_storage;
pub mod transform;

pub trait Component: Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static {}