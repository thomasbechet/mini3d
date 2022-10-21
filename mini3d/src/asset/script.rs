use slotmap::new_key_type;

use super::Asset;

pub struct RhaiScript {
    pub source: String,
}

new_key_type! { pub struct RhaiScriptId; }

impl Asset for RhaiScript {
    type Id = RhaiScriptId;
    fn typename() -> &'static str { "rhai_script" }
}