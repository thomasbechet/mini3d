use self::compiler::Compiler;

pub mod backend;
pub mod compiler;
pub mod export;
pub mod frontend;
pub mod interface;
pub mod interpreter;
pub mod mir;
pub mod module;

#[derive(Default)]
pub(crate) struct ScriptManager {
    compiler: Compiler,
}
