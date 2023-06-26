use self::compiler::Compiler;

pub mod compiler;
pub mod frontend;
pub mod interpreter;
pub mod mir;
pub mod module;

#[derive(Default)]
pub(crate) struct ScriptManager {
    compiler: Compiler,
}
