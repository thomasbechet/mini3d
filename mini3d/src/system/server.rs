use super::event::{ImportAssetEvent, SystemEvent};

pub trait SystemServer {
    fn poll_imports(&mut self) -> Option<ImportAssetEvent> {
        None
    }

    fn pool_events(&mut self) -> Option<SystemEvent> {
        None
    }

    fn request_stop(&mut self) {}
}
