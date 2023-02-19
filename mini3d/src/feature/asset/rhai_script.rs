use serde::{Serialize, Deserialize};

use crate::{registry::asset::Asset, uid::UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScript {
    pub source: String,
}

impl Asset for RhaiScript {}

impl RhaiScript {
    pub const NAME: &'static str = "rhai_script";
    pub const UID: UID = RhaiScript::NAME.into();
}