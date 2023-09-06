use mini3d_derive::Error;

use super::event::NetworkEvent;

#[derive(Debug, Error)]
pub enum NetworkServerError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait NetworkServer {
    fn events(&self) -> &[NetworkEvent] {
        &[]
    }
}
