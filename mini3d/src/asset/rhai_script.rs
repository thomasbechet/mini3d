use serde::{Serialize, Deserialize};
use super::Asset;

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScript {
    pub source: String,
}

impl Asset for RhaiScript {
    fn typename() -> &'static str { "rhai_script" }
}