use crate::platform::event::ImportAssetEvent;

use super::context::Context;

pub struct Runtime;

impl Runtime {
    pub fn next_import(ctx: &mut Context) -> Option<ImportAssetEvent> {
        ctx.runtime.next_import()
    }

    pub fn request_stop(ctx: &mut Context) {
        ctx.runtime.request_stop();
    }
}
