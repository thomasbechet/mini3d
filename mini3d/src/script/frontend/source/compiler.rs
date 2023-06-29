use crate::{
    context::asset::AssetContext,
    feature::asset::script::Script,
    registry::asset::Asset,
    script::{
        compiler::CompilationUnit,
        export::ExportTable,
        frontend::error::CompileError,
        module::{Module, ModuleId},
    },
};

use super::{
    ast::AST,
    lexer::Lexer,
    parser::{ASTParser, ImportExportParser},
    stream::SourceStream,
    strings::StringTable,
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
    pub(crate) fn resolve_cu_and_exports(
        &mut self,
        assets: &AssetContext,
        module: &mut Module,
        id: ModuleId,
        compilation_unit: &mut CompilationUnit,
        exports: &mut ExportTable,
    ) -> Result<(), CompileError> {
        // Build source stream
        let mut stream = SourceStream::new(
            &assets
                .get::<Script>(Script::UID, module.asset)
                .unwrap()
                .ok_or(CompileError::ScriptNotFound)?
                .source,
        );
        // Find imports and exports
        ImportExportParser::<SourceStream>::evaluate(
            &mut self.strings,
            &mut self.lexer,
            &mut stream,
            compilation_unit,
            Some(exports),
            id,
        )?;
        Ok(())
    }

    pub(crate) fn generate_mir(
        &mut self,
        assets: &AssetContext,
        exports: &ExportTable,
        module: &mut Module,
    ) -> Result<(), CompileError> {
        // Build source stream
        let mut stream = SourceStream::new(
            &assets
                .get::<Script>(Script::UID, module.asset)
                .unwrap()
                .ok_or(CompileError::ScriptNotFound)?
                .source,
        );
        // Generate AST
        ASTParser::<SourceStream>::evaluate(
            &mut self.ast,
            &mut self.symbols,
            &mut self.strings,
            &mut self.lexer,
            &mut stream,
        )?;
        self.ast.print();
        // Generate MIR
        Ok(())
    }
}
