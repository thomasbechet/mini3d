use mini3d_derive::Error;

use crate::{
    ecs::ECSManager,
    feature::{core::activity::ActivityDescriptorHandle, ecs::system::SystemSetHandle},
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
    resource::ResourceManager,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
    },
};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ActivityId(pub(crate) SlotId);

impl ActivityId {
    pub(crate) fn null() -> Self {
        Self(SlotId::null())
    }
}

pub(crate) struct ActivityEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) parent: ActivityId,
    pub(crate) ecs: SlotId,
}

pub(crate) struct ActivityUpdateContext<'a> {
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) system: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("progress")]
    Progress,
}

pub enum ActivityCommand {
    Start(ActivityId, ActivityDescriptorHandle),
    Stop(ActivityId),
    InjectSystemSet(ActivityId, SystemSetHandle),
}

#[derive(Default)]
pub(crate) struct ActivityManager {
    pub(crate) root: ActivityId,
    pub(crate) active: ActivityId,
    pub(crate) entries: SlotMap<ActivityEntry>,
    pub(crate) commands: Vec<ActivityCommand>,
}

impl ActivityManager {
    pub(crate) fn add(
        &mut self,
        name: &str,
        parent: ActivityId,
        descriptor: ActivityDescriptorHandle,
    ) -> ActivityId {
        let activity = ActivityId(self.entries.add(ActivityEntry {
            name: name.into(),
            parent,
            ecs: SlotId::null(),
        }));
        self.commands
            .push(ActivityCommand::Start(activity, descriptor));
        activity
    }

    pub(crate) fn remove(&mut self, activity: ActivityId) {
        self.commands.push(ActivityCommand::Stop(activity));
    }

    pub(crate) fn flush_commands(&mut self, ecs: &mut ECSManager, resource: &mut ResourceManager) {
        for command in self.commands.drain(..) {
            match command {
                ActivityCommand::Start(id, desc) => self.entries[id.0].ecs = ecs.add(id),
                ActivityCommand::Stop(id) => {
                    self.remove_entry(id);
                }
                ActivityCommand::InjectSystemSet(id, set) => todo!(),
            }
        }
    }

    fn remove_entry(&mut self, activity: ActivityId) {
        // Find childs
        let childs = self
            .entries
            .iter()
            .filter_map(|(id, e)| {
                if e.parent == activity {
                    Some(ActivityId(id))
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
            .entries
            .iter()
            .find_map(|(id, e)| if id == activity.0 { Some(id) } else { None })
            .unwrap();
        self.entries.remove(slot);
    }
}
