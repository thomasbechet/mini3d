use self::{
    event::{ImportAssetEvent, RuntimeEvent},
    provider::RuntimeProvider,
};

pub mod event;
pub mod provider;

#[derive(Default)]
pub struct RuntimeManager {
    provider: Box<dyn RuntimeProvider>,
    request_stop: bool,
}

impl RuntimeManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn RuntimeProvider>) {
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
                RuntimeEvent::RequestStop => self.request_stop = true,
            }
        }
        if self.request_stop {
            self.provider.request_stop();
            self.request_stop = false;
        }
    }
}
