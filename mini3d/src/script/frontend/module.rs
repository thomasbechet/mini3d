use crate::{
    asset::AssetManager, context::asset::AssetContext, feature::asset::script::Script,
    registry::asset::Asset, uid::UID,
};

use super::{
    error::CompileError,
    export::ExportTable,
    mir::{mir::MIR, MIRTable},
    node::compiler::NodeCompiler,
    source::{compiler::SourceCompiler, stream::SourceStream},
};

pub enum ModuleKind {
    Source,
    Node,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ModuleId(u32);

struct ModuleEntry {
    ident: UID,
    kind: ModuleKind,
    asset: UID,
}

#[derive(Default)]
pub(crate) struct ModuleTable {
    modules: Vec<ModuleEntry>,
}

impl ModuleTable {
    pub(crate) fn add(&mut self, ident: UID, kind: ModuleKind, asset: UID) -> ModuleId {
        let id = ModuleId(self.modules.len() as u32);
        self.modules.push(ModuleEntry { ident, kind, asset });
        id
    }

    pub(crate) fn find(&self, ident: UID) -> Option<ModuleId> {
        for (i, module) in self.modules.iter().enumerate() {
            if module.ident == ident {
                return Some(ModuleId(i as u32));
            }
        }
        None
    }

    pub(crate) fn resolve_exports_and_dependencies(
        &mut self,
        module: ModuleId,
        compilation_unit: &mut Vec<ModuleId>,
    ) -> Result<(), CompileError> {
        Ok(())
    }

    pub(crate) fn compile(
        &mut self,
        module: ModuleId,
        exports: &ExportTable,
        mirs: &mut MIRTable,
        source_compiler: &mut SourceCompiler,
        node_compiler: &mut NodeCompiler,
        assets: &AssetContext,
    ) -> Result<(), CompileError> {
        let mir = mirs.get_mut(module);
        let entry = self.modules.get_mut(module.0 as usize).unwrap();
        match entry.kind {
            ModuleKind::Source => {
                let script = assets
                    .get::<Script>(Script::UID, entry.asset)
                    .unwrap()
                    .ok_or(CompileError::ScriptNotFound)?;
                source_compiler.compile(exports, mir, &script.source)?;
            }
            ModuleKind::Node => {}
        }
        Ok(())
    }
}
