use anyhow::Result;

use crate::context::SystemContext;

pub type SystemCallback = fn(&mut SystemContext) -> Result<()>;
