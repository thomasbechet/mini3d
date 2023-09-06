pub mod event;
pub mod server;

#[derive(Default)]
pub struct SystemManager {
    request_stop: bool,
}
