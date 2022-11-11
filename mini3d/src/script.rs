use crate::rhai::RhaiScriptCache;

#[derive(Default)]
pub struct ScriptManager {
    pub rhai: RhaiScriptCache,
}