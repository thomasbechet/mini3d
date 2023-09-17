use self::{
    event::{ImportAssetEvent, SystemEvent},
    provider::SystemProvider,
};

pub mod event;
pub mod provider;

#[derive(Default)]
pub struct SystemManager {
    provider: Box<dyn SystemProvider>,
    request_stop: bool,
}

impl SystemManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn SystemProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    pub fn next_import(&mut self) -> Option<ImportAssetEvent> {
        self.provider.next_import()
    }

    pub(crate) fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                SystemEvent::RequestStop => self.request_stop = true,
            }
        }
        if self.request_stop {
            self.provider.request_stop();
            self.request_stop = false;
        }
    }
}
