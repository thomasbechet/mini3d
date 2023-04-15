use crate::rhai::RhaiScriptCache;

pub mod backend;
pub mod frontend;
pub mod interpreter;

#[derive(Default)]
pub(crate) struct ScriptManager {
    pub(crate) rhai: RhaiScriptCache,
}