use serde::{Serialize, Deserialize};
use slotmap::new_key_type;
use super::Asset;

new_key_type! { pub struct RhaiScriptId; }

#[derive(Serialize, Deserialize)]
pub struct RhaiScript {
    pub source: String,
}

impl Asset for RhaiScript {
    type Id = RhaiScriptId;
    fn typename() -> &'static str { "rhai_script" }
}