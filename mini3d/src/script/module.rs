use crate::uid::UID;

use super::interface::InterfaceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
    Source { asset: UID },
    Node { asset: UID },
    Interface { id: InterfaceId },
}

#[derive(Debug)]
struct ModuleEntry {
    uid: UID,
    module: Module,
}

#[derive(Default)]
pub(crate) struct ModuleTable {
    entries: Vec<ModuleEntry>,
}

impl ModuleTable {
    pub(crate) fn add(&mut self, uid: UID, module: Module) -> ModuleId {
        if let Some(id) = self.find(uid) {
            return id;
        }
        let id = ModuleId(self.entries.len() as u32);
        self.entries.push(ModuleEntry { uid, module });
        id
    }

    pub(crate) fn find(&self, uid: UID) -> Option<ModuleId> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.uid == uid {
                return Some(ModuleId(i as u32));
            }
        }
        None
    }

    pub(crate) fn get(&self, id: ModuleId) -> Option<&Module> {
        self.entries.get(id.0 as usize).map(|e| &e.module)
    }
}
