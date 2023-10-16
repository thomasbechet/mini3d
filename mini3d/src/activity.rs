use crate::{
    ecs::{
        container::ContainerTable, entity::EntityTable, query::QueryTable, scheduler::Scheduler,
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
    },
};

pub(crate) struct ActivityId(pub(crate) SlotId);

pub(crate) struct ActivityEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) parent: SlotId,
    pub(crate) containers: ContainerTable,
    entities: EntityTable,
    queries: QueryTable,
    instances: SystemInstanceTable,
    pub(crate) scheduler: Scheduler,
    global_cycle: u32,
}

pub(crate) struct ActivityManager {
    pub(crate) root: ActivityId,
    pub(crate) active: ActivityId,
    pub(crate) entries: SlotMap<ActivityEntry>,
}

impl Default for ActivityManager {
    fn default() -> Self {
        let mut manager = Self {
            root: ActivityId(SlotId::null()),
            active: ActivityId(SlotId::null()),
            entries: SlotMap::new(),
        };
        manager.root = ActivityId(manager.entries.add(ActivityEntry {
            name: AsciiArray::from("root"),
            parent: SlotId::null(),
            ecs: SlotId::null(),
        }));
        manager.active = manager.root;
        manager
    }
}

impl ActivityManager {
    pub(crate) fn add(&mut self, name: &str, parent: ActivityId) -> ActivityId {
        if self.entries.values().find(|e| e.name == name).is_some() {
            panic!("Duplicated activity name: {}", name);
        }
        let entry = ActivityEntry {
            name: AsciiArray::from(name),
            parent: parent.0,
            containers: ContainerTable::new(),
            entities: EntityTable::new(),
            queries: QueryTable::new(),
            instances: SystemInstanceTable::new(),
        };
        ActivityId(self.entries.add(entry))
    }

    pub(crate) fn remove(&mut self, activity: ActivityId) {
        let childs = self
            .entries
            .iter()
            .filter(|(_, e)| e.parent == activity.0)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        for child in childs {
            self.remove(child);
        }
        self.entries.remove(activity.0);
    }

    pub(crate) fn set_active(&mut self, activity: ActivityId) {
        self.active = activity;
    }
}
