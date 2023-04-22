use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum DiskBackendError {
    Unknown,
}

impl Error for DiskBackendError {}

impl Display for DiskBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskBackendError::Unknown => write!(f, "Unknown error"),
        }
    }
}

#[allow(unused_variables)]
pub trait DiskBackend {

}

#[derive(Default)]
pub struct DummyDiskBackend;

impl DiskBackend for DummyDiskBackend {}