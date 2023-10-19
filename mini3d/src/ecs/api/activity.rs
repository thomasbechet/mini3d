use crate::{activity::ActivityId, resource::handle::ResourceHandle};

use super::context::Context;

pub enum ActivityCommand {
    Start(ActivityId, ResourceHandle),
    Stop(ActivityId),
    InjectSystemSet(ActivityId, ResourceHandle),
}

pub(crate) struct ActivityContext {
    pub(crate) active: ActivityId,
    pub(crate) commands: Vec<ActivityCommand>,
    pub(crate) next_id: ActivityId,
}

impl Default for ActivityContext {
    fn default() -> Self {
        Self {
            active: Default::default(),
            commands: Default::default(),
            next_id: ActivityId(1),
        }
    }
}

pub struct Activity;

impl Activity {
    pub fn start(ctx: &mut Context, descriptor: ResourceHandle) -> ActivityId {
        let next = ctx.activity.next_id;
        ctx.activity.next_id.0 += 1;
        ctx.activity
            .commands
            .push(ActivityCommand::Start(next, descriptor));
        next
    }

    pub fn stop(ctx: &mut Context, id: ActivityId) {
        ctx.activity.commands.push(ActivityCommand::Stop(id));
    }

    pub fn active(ctx: &Context) -> ActivityId {
        ctx.activity
    }

    pub fn add_system_set(ctx: &mut Context, activity: ActivityId, set: ResourceHandle) {
        todo!()
    }

    pub fn remove_system_set(ctx: &mut Context, activity: ActivityId, set: ResourceHandle) {
        todo!()
    }
}
