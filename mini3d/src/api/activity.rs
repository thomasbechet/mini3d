use crate::{
    activity::ActivityHandle,
    feature::{core::activity::ActivityDescriptorHandle, ecs::system::SystemSetHandle},
};

use super::Context;

pub struct Activity;

impl Activity {
    pub fn start(
        ctx: &mut Context,
        name: &str,
        descriptor: ActivityDescriptorHandle,
    ) -> ActivityHandle {
        ctx.activity.start(name, ctx.activity.active, descriptor)
    }

    pub fn stop(ctx: &mut Context, activity: ActivityHandle) {
        ctx.activity.stop(activity);
    }

    pub fn active(ctx: &Context) -> ActivityHandle {
        ctx.activity.active
    }

    pub fn add_system_set(ctx: &mut Context, activity: ActivityHandle, set: SystemSetHandle) {
        ctx.activity.add_system_set(activity, set);
    }

    pub fn remove_system_set(ctx: &mut Context, activity: ActivityHandle, set: SystemSetHandle) {
        ctx.activity.remove_system_set(activity, set);
    }

    pub fn enable_system_set(
        ctx: &mut Context,
        activity: ActivityHandle,
        set: SystemSetHandle,
        enabled: bool,
    ) {
    }
}
