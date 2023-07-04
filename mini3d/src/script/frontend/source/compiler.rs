use crate::{
    context::asset::AssetContext,
    feature::asset::script::Script,
    registry::asset::Asset,
    script::{
        compiler::CompilationUnit,
        export::ExportTable,
        frontend::error::CompileError,
        mir::mir::MIR,
        module::{ModuleId, ModuleTable},
    },
    uid::UID,
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
    fn prepare(&mut self) {
        self.symbols.clear();
        self.ast.clear();
        self.strings.clear();
    }

    pub(crate) fn resolve_cu_and_exports(
        &mut self,
        assets: &AssetContext,
        asset: UID,
        modules: &ModuleTable,
        module: ModuleId,
        compilation_unit: &mut CompilationUnit,
        exports: &mut ExportTable,
    ) -> Result<(), CompileError> {
        // Build source stream
        let mut stream = SourceStream::new(
            &assets
                .get::<Script>(Script::UID, asset)
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
            exports,
            modules,
            module,
        )?;
        Ok(())
    }

    pub(crate) fn generate_mir(
        &mut self,
        exports: &ExportTable,
        assets: &AssetContext,
        asset: UID,
        modules: &ModuleTable,
        module: ModuleId,
        mir: &mut MIR,
    ) -> Result<(), CompileError> {
        // Prepare compiler
        self.prepare();
        // Build source stream
        let mut stream = SourceStream::new(
            &assets
                .get::<Script>(Script::UID, asset)
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
            exports,
            modules,
            module,
        )?;
        self.symbols.print(&self.strings);
        self.ast.print();
        // Generate MIR
        Ok(())
    }
}
