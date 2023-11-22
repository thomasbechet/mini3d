use mini3d_derive::Error;

use crate::{
    ecs::{ECSInstanceHandle, ECSManager},
    feature::{core::activity::ActivityHandle, ecs::system::SystemSetHandle},
    resource::ResourceManager,
    slot_map_key,
    utils::{slotmap::SlotMap, string::AsciiArray},
};

slot_map_key!(ActivityInstanceHandle);

pub(crate) struct ActivityEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) parent: ActivityInstanceHandle,
    pub(crate) ecs: ECSInstanceHandle,
}

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("progress")]
    Progress,
}

pub enum ActivityCommand {
    Start(ActivityInstanceHandle, ActivityHandle),
    Stop(ActivityInstanceHandle),
    AddSystemSet(ActivityInstanceHandle, SystemSetHandle),
    RemoveSystemSet(ActivityInstanceHandle, SystemSetHandle),
}

#[derive(Default)]
pub(crate) struct ActivityManager {
    pub(crate) root: ActivityInstanceHandle,
    pub(crate) active: ActivityInstanceHandle,
    pub(crate) activities: SlotMap<ActivityInstanceHandle, ActivityEntry>,
    pub(crate) commands: Vec<ActivityCommand>,
}

impl ActivityManager {
    pub(crate) fn start(
        &mut self,
        name: &str,
        parent: ActivityInstanceHandle,
        descriptor: ActivityHandle,
    ) -> ActivityInstanceHandle {
        let activity = self.activities.add(ActivityEntry {
            name: name.into(),
            parent,
            ecs: ECSInstanceHandle::null(),
        });
        self.commands
            .push(ActivityCommand::Start(activity, descriptor));
        activity
    }

    pub(crate) fn stop(&mut self, activity: ActivityInstanceHandle) {
        self.commands.push(ActivityCommand::Stop(activity));
    }

    pub(crate) fn add_system_set(
        &mut self,
        activity: ActivityInstanceHandle,
        set: SystemSetHandle,
    ) {
        self.commands
            .push(ActivityCommand::AddSystemSet(activity, set));
    }

    pub(crate) fn remove_system_set(
        &mut self,
        activity: ActivityInstanceHandle,
        set: SystemSetHandle,
    ) {
        self.commands
            .push(ActivityCommand::RemoveSystemSet(activity, set));
    }

    pub(crate) fn flush_commands(&mut self, ecs: &mut ECSManager, resource: &mut ResourceManager) {
        for command in self.commands.drain(..).collect::<Vec<_>>() {
            match command {
                ActivityCommand::Start(activity, desc) => {
                    self.activities[activity].ecs = ecs.add(activity)
                }
                ActivityCommand::Stop(activity) => {
                    self.remove_entry(activity);
                }
                ActivityCommand::AddSystemSet(activity, set) => {
                    let (ecs, systems) = &mut ecs.instances[self.activities[activity].ecs];
                    systems
                        .insert_system_set(
                            set,
                            &mut ecs.entities,
                            &mut ecs.queries,
                            &mut ecs.containers,
                            resource,
                        )
                        .expect("Failed to insert system set");
                    ecs.scheduler.rebuild(&systems, resource);
                }
                ActivityCommand::RemoveSystemSet(activity, set) => todo!(),
            }
        }
    }

    fn remove_entry(&mut self, activity: ActivityInstanceHandle) {
        // Find childs
        let childs = self
            .activities
            .iter()
            .filter_map(|(id, e)| if e.parent == activity { Some(id) } else { None })
            .collect::<Vec<_>>();
        // Remove childs recursively
        for child in childs {
            self.remove_entry(child);
        }
        // Remove activity
        let slot = self
            .activities
            .iter()
            .find_map(|(id, e)| if id == activity { Some(id) } else { None })
            .unwrap();
        self.activities.remove(slot);
    }
}
