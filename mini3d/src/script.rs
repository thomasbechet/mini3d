use self::compiler::Compiler;

pub mod compiler;
pub mod constant;
pub mod frontend;
pub mod interpreter;

#[derive(Default)]
pub(crate) struct ScriptManager {
    compiler: Compiler,
}
