use self::compiler::Compiler;

pub mod backend;
pub mod compiler;
pub mod frontend;
pub mod interface;
pub mod interpreter;
pub mod mir;
pub mod module;
pub mod reflection;

#[derive(Default)]
pub(crate) struct ScriptManager {
    programs: ProgramTable,
    compiler: Compiler,
}
