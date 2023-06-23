use self::mir::MIR;

use super::module::ModuleId;

pub mod binder;
pub mod mir;
pub mod primitive;
pub mod tac;

struct MIREntry {
    module: ModuleId,
    mir: MIR,
}

#[derive(Default)]
pub(crate) struct MIRTable {
    entries: Vec<MIREntry>,
}

impl MIRTable {
    pub(crate) fn get_mut(&mut self, module: ModuleId) -> &mut MIR {
        if let Some(i) = self.entries.iter().position(|e| e.module == module) {
            &mut self.entries[i].mir
        } else {
            let mir = MIR::default();
            self.entries.push(MIREntry { module, mir });
            &mut self.entries.last_mut().unwrap().mir
        }
    }
}
