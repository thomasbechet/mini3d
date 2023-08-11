use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum NetworkBackendError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait NetworkBackend {}
