use crate::{activity::ActivityId, resource::handle::ResourceHandle};

use super::context::Context;

pub struct Activity;

impl Activity {
    pub fn start(ctx: &mut Context, activity: ResourceHandle) -> ActivityId {
        todo!()
    }

    pub fn stop(ctx: &mut Context, activity: ActivityId) {
        todo!()
    }

    pub fn inject_system_set(ctx: &mut Context, activity: ActivityId, set: ResourceHandle) {
        todo!()
    }
}
