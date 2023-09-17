use super::event::{ImportAssetEvent, SystemEvent};

pub trait SystemProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn next_import(&mut self) -> Option<ImportAssetEvent>;
    fn next_event(&mut self) -> Option<SystemEvent>;
    fn request_stop(&mut self);
}

#[derive(Default)]
pub struct PassiveSystemProvider;

impl SystemProvider for PassiveSystemProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn next_import(&mut self) -> Option<ImportAssetEvent> {
        None
    }
    fn next_event(&mut self) -> Option<SystemEvent> {
        None
    }
    fn request_stop(&mut self) {}
}

impl Default for Box<dyn SystemProvider> {
    fn default() -> Self {
        Box::<PassiveSystemProvider>::default()
    }
}
