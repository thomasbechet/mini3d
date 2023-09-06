use super::event::ImportAssetEvent;

pub trait SystemServer {
    fn poll_imports(&mut self) -> Option<ImportAssetEvent> {
        None
    }

    fn request_stop(&self) -> bool {
        false
    }
}
