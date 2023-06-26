use crate::{
    context::asset::AssetContext, feature::asset::script::Script, registry::asset::Asset, uid::UID,
};

use super::{
    frontend::{
        error::CompileError,
        node::compiler::NodeCompiler,
        source::{compiler::SourceCompiler, stream::SourceStream},
    },
    module::{ModuleKind, ModuleTable},
};

#[derive(Default)]
pub struct Compiler {
    modules: ModuleTable,
    source_compiler: SourceCompiler,
    node_compiler: NodeCompiler,
}

impl Compiler {
    pub fn add_module(&mut self, kind: ModuleKind, asset: UID) {
        self.modules.add(asset, kind);
    }

    pub fn compile(&mut self, module: UID, assets: &mut AssetContext) -> Result<(), CompileError> {
        // Update database
        // TODO: load modules from asset database

        // Find module
        let module = self
            .modules
            .find(module)
            .ok_or(CompileError::ModuleNotFound)?;

        // Resolve dependencies and exports
        let mut unit = Vec::new();
        let mut i = 0;
        unit.push(module);
        while i < unit.len() {
            let module = unit.get(i).unwrap();
            let module = self.modules.get_mut(*module).unwrap();
            match module.kind {
                ModuleKind::Source => {
                    let script = assets
                        .get::<Script>(Script::UID, module.asset)
                        .unwrap()
                        .ok_or(CompileError::ScriptNotFound)?;
                    self.source_compiler.resolve_dependencies_and_exports(
                        &SourceStream::new(&script.source),
                        module,
                    )?;
                }
                ModuleKind::Node => {
                    unimplemented!()
                }
            }
            for dependency in module.dependencies.iter() {
                if !unit.contains(dependency) {
                    unit.push(*dependency);
                }
            }
            i += 1;
        }

        // Generate mid-level intermediate representations
        for module in unit.iter() {
            let module = self.modules.get_mut(*module).unwrap();
            match module.kind {
                ModuleKind::Source => {
                    let script = assets
                        .get::<Script>(Script::UID, module.asset)
                        .unwrap()
                        .ok_or(CompileError::ScriptNotFound)?;
                    self.source_compiler.generate_mir(module, &script.source)?;
                }
                ModuleKind::Node => {
                    unimplemented!()
                }
            }
        }

        // Generate program
        let kind = self.modules.get_mut(module).unwrap().kind;
        match kind {
            ModuleKind::Source => {
                self.source_compiler
                    .generate_program(module, &mut self.modules)?;
            }
            ModuleKind::Node => {
                unimplemented!()
            }
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
