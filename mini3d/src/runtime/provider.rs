use super::event::{ImportAssetEvent, RuntimeEvent};

pub trait RuntimeProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn next_import(&mut self) -> Option<ImportAssetEvent>;
    fn next_event(&mut self) -> Option<RuntimeEvent>;
    fn request_stop(&mut self);
}

#[derive(Default)]
pub struct PassiveSystemProvider;

impl RuntimeProvider for PassiveSystemProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn next_import(&mut self) -> Option<ImportAssetEvent> {
        None
    }
    fn next_event(&mut self) -> Option<RuntimeEvent> {
        None
    }
    fn request_stop(&mut self) {}
}

impl Default for Box<dyn RuntimeProvider> {
    fn default() -> Self {
        Box::<PassiveSystemProvider>::default()
    }
}
