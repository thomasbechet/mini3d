use mini3d_derive::Error;

use super::event::NetworkEvent;

#[derive(Debug, Error)]
pub enum NetworkProviderError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait NetworkProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn next_event(&mut self) -> Option<NetworkEvent>;
}

#[derive(Default)]
pub struct PassiveNetworkProvider;

impl NetworkProvider for PassiveNetworkProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn next_event(&mut self) -> Option<NetworkEvent> {
        None
    }
}
