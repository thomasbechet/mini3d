use alloc::boxed::Box;

use self::{
    event::{ImportAssetEvent, PlatformEvent},
    provider::PlatformProvider,
};

pub mod event;
pub mod provider;

#[derive(Default)]
pub struct PlatformManager {
    provider: Box<dyn PlatformProvider>,
    request_stop: bool,
}

impl PlatformManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn PlatformProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    pub(crate) fn next_import(&mut self) -> Option<ImportAssetEvent> {
        self.provider.next_import()
    }

    pub(crate) fn request_stop(&mut self) {
        self.request_stop = true;
    }

    pub(crate) fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                PlatformEvent::RequestStop => self.request_stop = true,
            }
        }
        if self.request_stop {
            self.provider.request_stop();
            self.request_stop = false;
        }
    }
}
