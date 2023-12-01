use alloc::boxed::Box;

use super::event::{ImportAssetEvent, PlatformEvent};

pub trait PlatformProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn next_import(&mut self) -> Option<ImportAssetEvent>;
    fn next_event(&mut self) -> Option<PlatformEvent>;
    fn request_stop(&mut self);
}

#[derive(Default)]
pub struct PassiveSystemProvider;

impl PlatformProvider for PassiveSystemProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn next_import(&mut self) -> Option<ImportAssetEvent> {
        None
    }
    fn next_event(&mut self) -> Option<PlatformEvent> {
        None
    }
    fn request_stop(&mut self) {}
}

impl Default for Box<dyn PlatformProvider> {
    fn default() -> Self {
        Box::<PassiveSystemProvider>::default()
    }
}
