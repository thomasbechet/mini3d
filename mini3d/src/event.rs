use mini3d_db::{
    database::{ComponentHandle, Database, GetComponentHandle},
    entity::Entity,
};
use mini3d_logger::{level::LogLevel, LoggerManager};
use mini3d_scheduler::{Scheduler, StageId};
use mini3d_utils::{
    slot_map_key,
    slotmap::{SecondaryMap, SlotMap},
    string::AsciiArray,
};

pub enum Event {
    Tick,
    Start,
    Stop,
    ComponentAdded(ComponentHandle),
    ComponentRemoved(ComponentHandle),
    User(UserEventHandle),
}

impl Event {
    pub fn component_added(c: impl GetComponentHandle) -> Self {
        Self::ComponentAdded(c.handle())
    }
    pub fn component_removed(c: impl GetComponentHandle) -> Self {
        Self::ComponentRemoved(c.handle())
    }
}

pub struct EventData {
    pub entity: Entity,
    pub component: ComponentHandle,
}

#[derive(Default)]
pub(crate) struct UserEventEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) stage: Option<StageId>,
    pub(crate) entity: Entity,
}

#[derive(Default)]
pub(crate) struct ComponentEventStages {
    pub(crate) added: Option<StageId>,
    pub(crate) removed: Option<StageId>,
}

slot_map_key!(UserEventHandle);

#[derive(Default)]
pub(crate) struct EventTable {
    pub(crate) user: SlotMap<UserEventHandle, UserEventEntry>,
    pub(crate) component: SecondaryMap<ComponentHandle, ComponentEventStages>,
    pub(crate) tick: Option<StageId>,
    pub(crate) start: Option<StageId>,
    pub(crate) stop: Option<StageId>,
}

impl EventTable {
    pub(crate) fn setup(&mut self, scheduler: &mut Scheduler) {
        self.tick = Some(scheduler.add_stage());
        self.start = Some(scheduler.add_stage());
        self.stop = Some(scheduler.add_stage());
    }

    pub(crate) fn find_user_event(&self, name: &str) -> Option<Entity> {
        self.user.iter().find_map(|(id, event)| {
            if event.name.as_str() == name {
                Some(event.entity)
            } else {
                None
            }
        })
    }

    pub(crate) fn add_user_event(
        &mut self,
        scheduler: &mut Scheduler,
        name: &str,
        e: Entity,
    ) -> UserEventHandle {
        if self.find_user_event(name).is_some() {
            panic!("Duplicated user event");
        }
        let stage = scheduler.add_stage();
        self.user.add(UserEventEntry {
            name: name.into(),
            stage: Some(stage),
            entity: e,
        })
    }

    pub(crate) fn remove_user_event(&mut self, scheduler: &mut Scheduler, id: UserEventHandle) {
        scheduler.remove_stage(self.user[id].stage.unwrap());
        self.user.remove(id)
    }

    pub(crate) fn get_stage_from_event(&self, event: Event) -> Option<StageId> {
        match event {
            Event::Tick => self.tick,
            Event::Start => self.start,
            Event::Stop => self.stop,
            Event::ComponentAdded(c) => self.component[c].added,
            Event::ComponentRemoved(c) => self.component[c].removed,
            Event::User(e) => todo!(),
        }
    }

    pub(crate) fn register_component_callbacks(
        &mut self,
        scheduler: &mut Scheduler,
        c: ComponentHandle,
    ) {
        let added = scheduler.add_stage();
        let removed = scheduler.add_stage();
        self.component.insert(
            c,
            ComponentEventStages {
                added: Some(added),
                removed: Some(removed),
            },
        );
    }

    pub(crate) fn unregister_component_callbacks(
        &mut self,
        scheduler: &mut Scheduler,
        c: ComponentHandle,
    ) {
        let stages = &self.component[c];
        scheduler.remove_stage(stages.added.unwrap());
        scheduler.remove_stage(stages.removed.unwrap());
        self.component.remove(c);
    }

    pub(crate) fn debug(&self, logger: &mut LoggerManager) {
        for stage in self.user.values() {
            logger.log(format_args!("STAGE {}", stage.name), LogLevel::Debug, None);
        }
    }
}
