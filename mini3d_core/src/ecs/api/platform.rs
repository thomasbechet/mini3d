use crate::platform::event::ImportAssetEvent;

use super::Context;

pub struct Platform;

impl Platform {
    pub fn next_import(ctx: &mut Context) -> Option<ImportAssetEvent> {
        ctx.platform.next_import()
    }

    pub fn request_stop(ctx: &mut Context) {
        ctx.platform.request_stop();
    }
}
