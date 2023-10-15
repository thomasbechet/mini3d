use crate::utils::{
    slotmap::{SlotId, SlotMap},
    string::AsciiArray,
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
    pub(crate) parent: SlotId,
    pub(crate) first_child: SlotId,
    pub(crate) next_sibling: SlotId,
    pub(crate) ecs: SlotId,
}

pub(crate) struct ActivityManager {
    root: ActivityId,
    pub(crate) entries: SlotMap<ActivityEntry>,
}

impl Default for ActivityManager {
    fn default() -> Self {
        let mut manager = Self {
            root: ActivityId(SlotId::null()),
            entries: SlotMap::new(),
        };
        // TODO: add root activity
        manager
    }
}

impl ActivityManager {
    pub(crate) fn add(&mut self, name: &str, parent: ActivityId) -> ActivityId {
        let entry = ActivityEntry {
            name: AsciiArray::from(name),
            status: ActivityStatus::Starting,
            parent: parent.0,
            first_child: None,
            next_sibling: None,
            ecs: SlotId::null(),
        };
        let id = self.entries.add(entry);
        if let Some(parent) = self.entries.get_mut(parent) {
            // Find last child
            let mut last = parent.first_child;
            while !last.is_null() {
                if self.entries[last].next_sibling.is_null() {
                    break;
                }
                last = self.entries[last].next_sibling;
            }
            // Append to child
            if last.is_null() {
                self.entries[last].next_sibling = id;
            }
        }
        ActivityId(id)
    }
}
