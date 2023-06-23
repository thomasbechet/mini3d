use self::compiler::Compiler;

pub mod compiler;
pub mod frontend;
pub mod interpreter;
pub mod string;

#[derive(Default)]
pub(crate) struct ScriptManager {
    compiler: Compiler,
}
