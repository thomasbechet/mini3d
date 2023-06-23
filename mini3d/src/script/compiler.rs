use crate::{context::asset::AssetContext, uid::UID};

use super::frontend::{
    error::CompileError,
    export::ExportTable,
    mir::MIRTable,
    module::{ModuleId, ModuleKind, ModuleTable},
    node::compiler::NodeCompiler,
    source::compiler::SourceCompiler,
};

#[derive(Default)]
pub struct Compiler {
    modules: ModuleTable,
    exports: ExportTable,
    mirs: MIRTable,
    source_compiler: SourceCompiler,
    node_compiler: NodeCompiler,
    compilation_unit: Vec<ModuleId>,
}

impl Compiler {
    pub fn add_module(&mut self, ident: UID, kind: ModuleKind, asset: UID) {
        self.modules.add(ident, kind, asset);
    }

    pub fn compile(&mut self, module: UID, assets: &mut AssetContext) -> Result<(), CompileError> {
        // Update database

        // Resolve dependencies and exports
        self.compilation_unit.clear();
        self.compilation_unit.push(
            self.modules
                .find(module)
                .ok_or(CompileError::ModuleNotFound)?,
        );
        let mut i = 0;
        while i < self.compilation_unit.len() {
            let module = self.compilation_unit.get(i).unwrap();
            self.modules
                .resolve_exports_and_dependencies(*module, &mut self.compilation_unit)?;
            i += 1;
        }

        // Compile
        for module in self.compilation_unit.iter() {
            self.modules.compile(
                *module,
                &self.exports,
                &mut self.mirs,
                &mut self.source_compiler,
                &mut self.node_compiler,
                assets,
            )?;
        }

        Ok(())

        // Propagate constant exports

        // // Lexical analysis
        // // Syntax analysis
        // SyntaxAnalysis::evaluate(
        //     &mut self.ast,
        //     &mut self.symtab,
        //     &mut self.strings,
        //     &mut self.lexer,
        //     source,
        // )?;
        // self.ast.print();
        // self.symtab.print(&self.strings);
        // self.strings.print();
        // // Semantic analysis
        // // SemanticAnalysis::check_undefined_symbols(&self.symtab)?;
        // SemanticAnalysis::infer_function_return_types(&mut self.symtab, &mut self.ast);
        // // Code generation
        // // Optimization
        // Ok(Program::empty())
    }
}
