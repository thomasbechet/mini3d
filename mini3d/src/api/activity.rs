use crate::{
    activity::ActivityId,
    feature::{core::activity::ActivityDescriptorHandle, ecs::system::SystemSetHandle},
};

use super::Context;

pub struct Activity;

impl Activity {
    pub fn start(
        ctx: &mut Context,
        name: &str,
        descriptor: ActivityDescriptorHandle,
    ) -> ActivityId {
        ctx.activity.add(name, ctx.activity.active, descriptor)
    }

    pub fn stop(ctx: &mut Context, activity: ActivityId) {
        ctx.activity.remove(activity);
    }

    pub fn active(ctx: &Context) -> ActivityId {
        ctx.activity.active
    }

    pub fn add_system_set(ctx: &mut Context, activity: ActivityId, set: SystemSetHandle) {
        todo!()
    }

    pub fn remove_system_set(ctx: &mut Context, activity: ActivityId, set: SystemSetHandle) {
        todo!()
    }
}
