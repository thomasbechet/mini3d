use std::error::Error;

use crate::context::SystemContext;

pub type SystemResult = Result<(), Box<dyn Error>>;

pub type SystemCallback = fn(&mut SystemContext) -> SystemResult;
