use crate::rhai::RhaiScriptCache;

pub mod vm;

#[derive(Default)]
pub(crate) struct ScriptManager {
    pub(crate) rhai: RhaiScriptCache,
}