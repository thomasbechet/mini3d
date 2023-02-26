use anyhow::Result;

use crate::{uid::UID, feature::asset::system_group::SystemGroup, ecs::scheduler::Scheduler};

pub struct SchedulerContext<'a> {
    pub(crate) scheduler: &'a mut Scheduler,
}

impl<'a> SchedulerContext<'a> {

    pub fn add_group(&mut self, name: &str, group: SystemGroup) -> Result<UID> {
        self.scheduler.add_group(name, group)
    }

    pub fn remove_group(&mut self, group: UID) -> Result<()> {
        self.scheduler.remove_group(group)
    }

    pub fn enable_group(&mut self, group: UID) -> Result<()> {
        self.scheduler.enable_group(group)
    }

    pub fn disable_group(&mut self, group: UID) -> Result<()> {
        self.scheduler.disable_group(group)
    }
}