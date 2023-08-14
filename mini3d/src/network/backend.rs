use mini3d_derive::Error;

use super::event::NetworkEvent;

#[derive(Debug, Error)]
pub enum NetworkBackendError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait NetworkBackend {
    fn events(&self) -> &[NetworkEvent] {
        &[]
    }
}