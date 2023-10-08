use crate::{
    ecs::ECSManager,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
    },
};

pub(crate) struct ProgramId(pub(crate) SlotId);

#[derive(Default)]
pub(crate) enum ProgramStatus {
    #[default]
    Starting,
    Running,
    Stopping,
}

pub(crate) const MAX_PROGRAM_NAME_LEN: usize = 32;

pub(crate) struct ProgramEntry {
    pub(crate) name: AsciiArray<MAX_PROGRAM_NAME_LEN>,
    pub(crate) status: ProgramStatus,
    pub(crate) parent: ProgramId,
    pub(crate) ecs: ECSManager,
}

pub(crate) struct ProgramManager {
    pub(crate) entries: SlotMap<ProgramEntry>,
}

impl Default for ProgramManager {
    fn default() -> Self {
        Self {
            entries: SlotMap::new(),
        }
    }
}
