use crate::uid::UID;

use super::{mir::primitive::PrimitiveType, module::ModuleId};

pub(crate) type ExportId = u32;

pub(crate) enum Export {
    Function {
        ty: PrimitiveType,
        first_arg: Option<ExportId>,
    },
    FunctionArgument {
        ty: PrimitiveType,
        next_arg: Option<ExportId>,
    },
    Constant {
        ty: PrimitiveType,
    },
}

pub struct ExportEntry {
    module: ModuleId,
    name: UID,
    export: Export,
}

#[derive(Default)]
pub(crate) struct ExportTable {
    entries: Vec<ExportEntry>,
}

impl ExportTable {
    pub(crate) fn add(&mut self, module: ModuleId, name: UID, export: Export) -> ExportId {
        let id = self.entries.len() as ExportId;
        self.entries.push(ExportEntry {
            module,
            name,
            export,
        });
        id
    }

    pub(crate) fn get(&self, id: ExportId) -> Option<&Export> {
        self.entries.get(id as usize).map(|entry| &entry.export)
    }

    pub(crate) fn find(&self, module: ModuleId, name: UID) -> Option<ExportId> {
        self.entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.module == module && entry.name == name)
            .map(|(id, _)| id as ExportId)
    }
}
