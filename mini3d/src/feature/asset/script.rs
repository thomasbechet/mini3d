use serde::{Serialize, Deserialize};

use crate::{registry::asset::Asset, uid::UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct Script {
    pub source: String,
}

impl Asset for Script {}

impl Script {
    pub const NAME: &'static str = "script";
    pub const UID: UID = UID::new(Script::NAME);
}