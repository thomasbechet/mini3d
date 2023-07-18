use crate::event::{asset::ImportAssetEvent, Events};

pub struct EventContext<'a> {
    pub(crate) events: &'a Events,
}

impl<'a> EventContext<'a> {
    pub fn import_asset(&self) -> &[ImportAssetEvent] {
        &self.events.asset
    }
}
