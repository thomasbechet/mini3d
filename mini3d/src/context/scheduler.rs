use std::cell::{RefMut, RefCell};

use anyhow::Result;

use crate::{uid::UID, feature::asset::system_group::SystemGroup, ecs::scheduler::Scheduler};

pub struct SchedulerContext<'a> {
    scheduler: RefMut<'a, Scheduler>,
}

impl<'a> SchedulerContext<'a> {

    pub(crate) fn new(scheduler: &'a RefCell<Scheduler>) -> Self {
        Self { scheduler: scheduler.borrow_mut() }
    }

    pub fn add_group(&mut self, name: &str, group: &SystemGroup) -> Result<UID> {
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