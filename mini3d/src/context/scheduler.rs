use crate::{
    ecs::{error::SchedulerError, scheduler::Scheduler},
    uid::UID,
};

pub struct SchedulerContext<'a> {
    pub(crate) scheduler: &'a mut Scheduler,
}

impl<'a> SchedulerContext<'a> {
    /// Applied at the end of the procedure
    pub fn add_group(&mut self, name: &str, group: SystemGroup) -> Result<UID, SchedulerError> {
        self.scheduler.add_group(name, group)
    }

    /// Applied at the end of the procedure
    pub fn remove_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.scheduler.remove_group(group)
    }

    /// Applied at the end of the procedure
    pub fn enable_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.scheduler.enable_group(group)
    }

    /// Applied at the end of the procedure
    pub fn disable_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.scheduler.disable_group(group)
    }
}
