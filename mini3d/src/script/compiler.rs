use crate::{
    context::asset::AssetContext, feature::asset::script::Script, registry::asset::Asset, uid::UID,
};

use super::{
    backend::compiler::BackendCompiler,
    export::ExportTable,
    frontend::{
        error::CompileError, node::compiler::NodeCompiler, source::compiler::SourceCompiler,
    },
    module::{ModuleId, ModuleKind, ModuleTable},
};

#[derive(Default)]
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
    exports: ExportTable,
    compilation_unit: CompilationUnit,
    source_compiler: SourceCompiler,
    node_compiler: NodeCompiler,
    backend_compiler: BackendCompiler,
}

impl Compiler {
    pub fn add_module(&mut self, kind: ModuleKind, asset: UID) {
        self.modules.add(asset, kind);
    }

    fn prepare(&mut self) {}

    fn fetch_modules(&mut self, assets: &AssetContext) -> Result<(), CompileError> {
        Ok(())
    }

    fn resolve_cu_and_exports(
        &mut self,
        entry: UID,
        assets: &AssetContext,
    ) -> Result<(), CompileError> {
        // Insert entry module
        let entry = self
            .modules
            .find(entry)
            .ok_or(CompileError::ModuleNotFound)?;
        self.compilation_unit.add(entry);
        let mut i = 0;
        while i < self.compilation_unit.len() {
            let module = self.modules.get_mut(self.compilation_unit.get(i)).unwrap();
            match module.kind {
                ModuleKind::Source => self.source_compiler.resolve_cu_and_exports(
                    assets,
                    module,
                    &mut self.compilation_unit,
                    &mut self.exports,
                )?,
                ModuleKind::Node => unimplemented!(),
            }
            i += 1;
        }
        Ok(())
    }

    fn generate_mirs(&mut self, assets: &AssetContext) -> Result<(), CompileError> {
        for module in self.compilation_unit.modules.iter() {
            let module = self.modules.get_mut(*module).unwrap();
            match module.kind {
                ModuleKind::Source => {
                    self.source_compiler
                        .generate_mir(assets, &self.exports, module)?
                }
                ModuleKind::Node => unimplemented!(),
            }
        }
        Ok(())
    }

    fn generate_program(&mut self, entry: UID) -> Result<(), CompileError> {
        Ok(())
    }

    pub fn compile(&mut self, module: UID, assets: &AssetContext) -> Result<(), CompileError> {
        // Reset compiler resources
        self.prepare();
        // Fetch all modules from the asset manager (sequential, acquire cached modules)
        self.fetch_modules(assets)?;
        // Resolve compilation unit and exports (sequential, fast if cached)
        self.resolve_cu_and_exports(module, assets)?;
        // Generate MIRs for all modules in the compilation unit (parallel, fast if cached)
        self.generate_mirs(assets)?;
        // Generate program (sequential, slow)
        self.generate_program(module)?;
        Ok(())
    }
}
