use crate::uid::UID;

use super::mir::mir::MIR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleKind {
    Source,
    Node,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ModuleId(u32);

pub(crate) struct Module {
    pub(crate) kind: ModuleKind,
    pub(crate) asset: UID,
    pub(crate) mir: MIR,
}

#[derive(Default)]
pub(crate) struct ModuleTable {
    modules: Vec<Module>,
}

impl ModuleTable {
    pub(crate) fn add(&mut self, asset: UID, kind: ModuleKind) -> ModuleId {
        let id = ModuleId(self.modules.len() as u32);
        self.modules.push(Module {
            kind,
            asset,
            mir: Default::default(),
        });
        id
    }

    pub(crate) fn find(&self, uid: UID) -> Option<ModuleId> {
        for (i, module) in self.modules.iter().enumerate() {
            if module.asset == uid {
                return Some(ModuleId(i as u32));
            }
        }
        None
    }

    pub(crate) fn get_mut(&mut self, module: ModuleId) -> Option<&mut Module> {
        self.modules.get_mut(module.0 as usize)
    }
}
