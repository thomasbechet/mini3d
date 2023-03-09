use crate::rhai::RhaiScriptCache;

#[derive(Default)]
pub(crate) struct ScriptManager {
    pub(crate) rhai: RhaiScriptCache,
}