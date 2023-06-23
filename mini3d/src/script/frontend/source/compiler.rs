use crate::script::{
    frontend::{
        error::CompileError,
        export::ExportTable,
        mir::mir::MIR,
        module::{ModuleId, ModuleTable},
    },
    string::StringTable,
};

use super::{
    ast::AST, lexer::Lexer, parser::Parser, stream::SourceStream, symbol::SourceSymbolTable,
};

#[derive(Default)]
pub(crate) struct SourceCompiler {
    symbols: SourceSymbolTable,
    ast: AST,
    lexer: Lexer,
    strings: StringTable,
}

impl SourceCompiler {
    pub(crate) fn collect_dependencies_and_exports(
        &mut self,
        stream: &SourceStream,
        exports: &mut ExportTable,
        modules: &mut ModuleTable,
        compilation_unit: &mut Vec<ModuleId>,
    ) -> Result<(), CompileError> {
        Ok(())
    }

    pub(crate) fn compile(
        &mut self,
        exports: &ExportTable,
        mir: &mut MIR,
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
