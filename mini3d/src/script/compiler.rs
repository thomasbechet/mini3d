use crate::utils::uid::UID;

use super::{
    backend::compiler::BackendCompiler,
    frontend::{
        error::CompileError, node::compiler::NodeCompiler, source::compiler::SourceCompiler,
    },
    interface::InterfaceTable,
    mir::MIRTable,
    module::{Module, ModuleId, ModuleTable},
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
    interfaces: InterfaceTable,
    mirs: MIRTable,
    compilation_unit: CompilationUnit,
    source_compiler: SourceCompiler,
    node_compiler: NodeCompiler,
    backend_compiler: BackendCompiler,
}

impl Compiler {
    pub fn add_module(&mut self, uid: UID, module: Module) -> ModuleId {
        let module = self.modules.add(uid, module);
        self.mirs.add(module);
        module
    }

    fn fetch_modules(&mut self, assets: &AssetContext) -> Result<(), CompileError> {
        // Insert builtin modules
        self.modules.add("scene".into(), Module::Builtin);
        self.modules.add("asset".into(), Module::Builtin);
        self.modules.add("input".into(), Module::Builtin);
        self.modules.add("renderer".into(), Module::Builtin);
        self.modules.add("physics".into(), Module::Builtin);
        self.modules.add("registry".into(), Module::Builtin);
        self.modules.add("math".into(), Module::Builtin);
        Ok(())
    }

    fn resolve_cu_and_exports(
        &mut self,
        entry: ModuleId,
        assets: &AssetContext,
    ) -> Result<(), CompileError> {
        println!("=> Resolve CU and exports");
        // Insert entry module
        self.compilation_unit.add(entry);
        let mut i = 0;
        while i < self.compilation_unit.len() {
            let module = self.compilation_unit.get(i);
            match self.modules.get(module).unwrap() {
                Module::Source { asset } => self.source_compiler.resolve_cu_and_exports(
                    assets,
                    *asset,
                    &mut self.modules,
                    module,
                    &mut self.compilation_unit,
                )?,
                Module::Node { .. } => unimplemented!(),
                Module::Interface { .. } => unimplemented!(),
                Module::Builtin { .. } => unimplemented!(),
            }
            i += 1;
        }
        Ok(())
    }

    fn generate_mirs(&mut self, assets: &AssetContext) -> Result<(), CompileError> {
        println!("=> Generate MIRs");
        for module in self.compilation_unit.modules.iter() {
            let mir = self.mirs.get_mut(*module).unwrap();
            match self.modules.get(*module).unwrap() {
                Module::Source { asset } => self.source_compiler.generate_mir(
                    assets,
                    *asset,
                    &self.modules,
                    *module,
                    mir,
                )?,
                Module::Node { .. } => unimplemented!(),
                Module::Interface { .. } => unimplemented!(),
                Module::Builtin { .. } => unimplemented!(),
            }
        }
        Ok(())
    }

    fn generate_program(&mut self, entry: ModuleId) -> Result<(), CompileError> {
        println!("=> Generate program");
        Ok(())
    }

    pub fn compile(
        &mut self,
        entry: ModuleId,
        assets: &AssetContext,
        registry: &RegistryContext,
    ) -> Result<(), CompileError> {
        // Fetch all modules from the asset manager (sequential, acquire cached modules)
        self.fetch_modules(assets)?;
        // Resolve compilation unit and exports (sequential, fast if cached)
        self.resolve_cu_and_exports(entry, assets)?;
        self.modules.print();
        // Generate MIRs for all modules in the compilation unit (parallel, fast if cached)
        self.generate_mirs(assets)?;
        // Generate program (sequential, slow)
        self.generate_program(entry)?;
        Ok(())
    }
}
