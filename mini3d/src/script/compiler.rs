use crate::{
    context::asset::AssetContext, feature::asset::script::Script, registry::asset::Asset, uid::UID,
};

use super::{
    backend::compiler::BackendCompiler,
    frontend::{
        error::CompileError, node::compiler::NodeCompiler, source::compiler::SourceCompiler,
    },
    module::{ModuleId, ModuleKind, ModuleTable},
};

pub(crate) struct CompilationUnit {
    modules: Vec<ModuleId>,
}

impl CompilationUnit {
    pub(crate) fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub(crate) fn add(&mut self, module: ModuleId) {
        if !self.modules.contains(&module) {
            self.modules.push(module);
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.modules.len()
    }

    pub(crate) fn get(&self, index: usize) -> ModuleId {
        *self.modules.get(index).unwrap()
    }
}

#[derive(Default)]
pub struct Compiler {
    modules: ModuleTable,
    source_compiler: SourceCompiler,
    node_compiler: NodeCompiler,
    backend_compiler: BackendCompiler,
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

        // Generate MIRs
        let mut compilation_unit = CompilationUnit::new();
        compilation_unit.add(module);
        let mut i = 0;
        while i < compilation_unit.len() {
            let module = compilation_unit.get(i);
            let module = self.modules.get_mut(module).unwrap();
            match module.kind {
                ModuleKind::Source => {
                    let script = assets
                        .get::<Script>(Script::UID, module.asset)
                        .unwrap()
                        .ok_or(CompileError::ScriptNotFound)?;
                    self.source_compiler.generate_mir(
                        module,
                        &mut compilation_unit,
                        &script.source,
                    )?;
                }
                ModuleKind::Node => {
                    unimplemented!()
                }
            }
            i += 1;
        }

        // Resolve constants

        // Optimize MIRs

        // Generate program

        Ok(())
    }
}
