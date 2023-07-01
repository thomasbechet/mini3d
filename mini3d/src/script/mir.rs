use self::mir::MIR;

use super::module::ModuleId;

pub mod data;
pub mod instruction;
pub mod mir;
pub mod primitive;
pub mod slotmap;

pub(crate) struct MIREntry {
    mir: MIR,
    module: ModuleId,
}

#[derive(Default)]
pub(crate) struct MIRTable {
    entries: Vec<MIREntry>,
}

impl MIRTable {
    pub(crate) fn add(&mut self, module: ModuleId) {
        self.entries.push(MIREntry {
            mir: MIR::default(),
            module,
        });
    }

    pub(crate) fn get_mut(&mut self, module: ModuleId) -> Option<&mut MIR> {
        for entry in self.entries.iter_mut() {
            if entry.module == module {
                return Some(&mut entry.mir);
            }
        }
        None
    }
}
