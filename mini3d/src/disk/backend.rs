use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum DiskBackendError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait DiskBackend {

}

#[derive(Default)]
pub struct DummyDiskBackend;

impl DiskBackend for DummyDiskBackend {}