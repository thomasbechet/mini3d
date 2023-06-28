use crate::script::{
    compiler::CompilationUnit,
    frontend::error::CompileError,
    interpreter::program::Program,
    module::{Module, ModuleId, ModuleTable},
};

use super::{
    ast::AST, lexer::Lexer, parser::Parser, stream::SourceStream, strings::StringTable,
    symbol::SymbolTable,
};

pub(crate) struct SourceCompiler {
    symbols: SymbolTable,
    ast: AST,
    lexer: Lexer,
    strings: StringTable,
}

impl Default for SourceCompiler {
    fn default() -> Self {
        Self {
            symbols: Default::default(),
            ast: Default::default(),
            lexer: Lexer::new(false),
            strings: Default::default(),
        }
    }
}

impl SourceCompiler {
    pub(crate) fn generate_mir(
        &mut self,
        module: &mut Module,
        compilation_unit: &mut CompilationUnit,
        source: &str,
    ) -> Result<(), CompileError> {
        // Generate AST
        Parser::<SourceStream>::evaluate(
            &mut self.ast,
            &mut self.symbols,
            &mut self.strings,
            &mut self.lexer,
            &mut SourceStream::new(source),
        )?;
        self.ast.print();
        // Generate MIR
        Ok(())
    }
}
