use crate::{
    ecs::ECSManager,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
    },
};

pub(crate) struct ActivityId(pub(crate) SlotId);

#[derive(Default)]
pub(crate) enum ActivityStatus {
    #[default]
    Starting,
    Running,
    Stopping,
}

pub(crate) const MAX_ACTIVITY_NAME_LEN: usize = 32;

pub(crate) struct ActivityEntry {
    pub(crate) name: AsciiArray<MAX_ACTIVITY_NAME_LEN>,
    pub(crate) status: ActivityStatus,
    pub(crate) parent: ActivityId,
    pub(crate) ecs: ECSManager,
}

pub(crate) struct ActivityManager {
    pub(crate) entries: SlotMap<ActivityEntry>,
}

impl Default for ActivityManager {
    fn default() -> Self {
        Self {
            entries: SlotMap::new(),
        }
    }
}
