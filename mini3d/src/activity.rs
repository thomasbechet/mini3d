use mini3d_derive::Error;

use crate::{
    ecs::ECSManager,
    feature::{core::activity::ActivityDescriptorHandle, ecs::system::SystemSetHandle},
    resource::ResourceManager,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
    },
};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivityHandle(pub(crate) SlotId);

impl ActivityHandle {
    pub fn null() -> Self {
        Self(SlotId::null())
    }
}

pub(crate) struct ActivityEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) parent: ActivityHandle,
    pub(crate) ecs: SlotId,
}

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("progress")]
    Progress,
}

pub enum ActivityCommand {
    Start(ActivityHandle, ActivityDescriptorHandle),
    Stop(ActivityHandle),
    AddSystemSet(ActivityHandle, SystemSetHandle),
    RemoveSystemSet(ActivityHandle, SystemSetHandle),
}

#[derive(Default)]
pub(crate) struct ActivityManager {
    pub(crate) root: ActivityHandle,
    pub(crate) active: ActivityHandle,
    pub(crate) activities: SlotMap<ActivityEntry>,
    pub(crate) commands: Vec<ActivityCommand>,
}

impl ActivityManager {
    pub(crate) fn start(
        &mut self,
        name: &str,
        parent: ActivityHandle,
        descriptor: ActivityDescriptorHandle,
    ) -> ActivityHandle {
        let activity = ActivityHandle(self.activities.add(ActivityEntry {
            name: name.into(),
            parent,
            ecs: SlotId::null(),
        }));
        self.commands
            .push(ActivityCommand::Start(activity, descriptor));
        activity
    }

    pub(crate) fn stop(&mut self, activity: ActivityHandle) {
        self.commands.push(ActivityCommand::Stop(activity));
    }

    pub(crate) fn add_system_set(&mut self, activity: ActivityHandle, set: SystemSetHandle) {
        self.commands
            .push(ActivityCommand::AddSystemSet(activity, set));
    }

    pub(crate) fn remove_system_set(&mut self, activity: ActivityHandle, set: SystemSetHandle) {
        self.commands
            .push(ActivityCommand::RemoveSystemSet(activity, set));
    }

    pub(crate) fn flush_commands(&mut self, ecs: &mut ECSManager, resource: &mut ResourceManager) {
        for command in self.commands.drain(..).collect::<Vec<_>>() {
            match command {
                ActivityCommand::Start(activity, desc) => {
                    self.activities[activity.0].ecs = ecs.add(activity)
                }
                ActivityCommand::Stop(activity) => {
                    self.remove_entry(activity);
                }
                ActivityCommand::AddSystemSet(activity, set) => {
                    let instance = &mut ecs.instances[self.activities[activity.0].ecs];
                    instance
                        .systems
                        .insert_system_set(
                            set,
                            &mut instance.entities,
                            &mut instance.queries,
                            &mut instance.containers,
                            resource,
                        )
                        .expect("Failed to insert system set");
                    instance.scheduler.rebuild(&instance.systems, resource);
                }
                ActivityCommand::RemoveSystemSet(activity, set) => todo!(),
            }
        }
    }

    fn remove_entry(&mut self, activity: ActivityHandle) {
        // Find childs
        let childs = self
            .activities
            .iter()
            .filter_map(|(id, e)| {
                if e.parent == activity {
                    Some(ActivityHandle(id))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        // Remove childs recursively
        for child in childs {
            self.remove_entry(child);
        }
        // Remove activity
        let slot = self
            .activities
            .iter()
            .find_map(|(id, e)| if id == activity.0 { Some(id) } else { None })
            .unwrap();
        self.activities.remove(slot);
    }
}
