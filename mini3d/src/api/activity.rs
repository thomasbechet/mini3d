use crate::{
    activity::ActivityInstanceHandle,
    feature::{core::activity::ActivityHandle, ecs::system::SystemSetHandle},
};

use super::Context;

pub struct Activity;

impl Activity {
    pub fn start(
        ctx: &mut Context,
        name: &str,
        descriptor: ActivityHandle,
    ) -> ActivityInstanceHandle {
        ctx.activity.start(name, ctx.activity.active, descriptor)
    }

    pub fn stop(ctx: &mut Context, activity: ActivityInstanceHandle) {
        ctx.activity.stop(activity);
    }

    pub fn active(ctx: &Context) -> ActivityInstanceHandle {
        ctx.activity.active
    }

    pub fn add_system_set(
        ctx: &mut Context,
        activity: ActivityInstanceHandle,
        set: SystemSetHandle,
    ) {
        ctx.activity.add_system_set(activity, set);
    }

    pub fn remove_system_set(
        ctx: &mut Context,
        activity: ActivityInstanceHandle,
        set: SystemSetHandle,
    ) {
        ctx.activity.remove_system_set(activity, set);
    }

    pub fn enable_system_set(
        ctx: &mut Context,
        activity: ActivityInstanceHandle,
        set: SystemSetHandle,
        enabled: bool,
    ) {
    }

    pub fn set_target_fps(ctx: &mut Context, activity: ActivityInstanceHandle, fps: u16) {
        ctx.activity.set_target_fps(activity, fps);
    }
}
