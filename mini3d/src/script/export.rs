use crate::uid::UID;

use super::{mir::primitive::PrimitiveType, module::ModuleId};

pub(crate) type ExportId = u32;

#[derive(Debug)]
pub(crate) enum Export {
    Function {
        name: UID,
        ty: PrimitiveType,
        first_arg: Option<ExportId>,
    },
    FunctionArgument {
        name: UID,
        ty: PrimitiveType,
        next_arg: Option<ExportId>,
    },
    Constant {
        name: UID,
        ty: PrimitiveType,
    },
}

pub struct ExportEntry {
    module: ModuleId,
    export: Export,
}

#[derive(Default)]
pub(crate) struct ExportTable {
    entries: Vec<ExportEntry>,
}

impl ExportTable {
    pub(crate) fn add(&mut self, module: ModuleId, export: Export) -> ExportId {
        let id = self.entries.len() as ExportId;
        self.entries.push(ExportEntry { module, export });
        id
    }

    pub(crate) fn get(&self, id: ExportId) -> Option<&Export> {
        self.entries.get(id as usize).map(|entry| &entry.export)
    }

    pub(crate) fn get_mut(&mut self, id: ExportId) -> Option<&mut Export> {
        self.entries
            .get_mut(id as usize)
            .map(|entry| &mut entry.export)
    }

    pub(crate) fn iter(&self, module: ModuleId) -> impl Iterator<Item = ExportId> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter(move |(_, entry)| entry.module == module)
            .map(|(id, _)| id as ExportId)
    }

    pub(crate) fn find(&self, module: ModuleId, name: UID) -> Option<ExportId> {
        self.entries
            .iter()
            .enumerate()
            .find(|(_, entry)| {
                entry.module == module
                    && match entry.export {
                        Export::Function { name: n, .. } | Export::Constant { name: n, .. } => {
                            n == name
                        }
                        _ => false,
                    }
            })
            .map(|(id, _)| id as ExportId)
    }

    pub(crate) fn prepare(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn print(&self) {
        println!("EXPORTS:");
        for (id, entry) in self.entries.iter().enumerate() {
            println!("- [{}]: {:?}", id, entry.export);
        }
    }
}
