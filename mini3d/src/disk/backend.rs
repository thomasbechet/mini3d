use mini3d_derive::Error;

use crate::serialize::{Encoder, Serialize};

#[derive(Debug, Error)]
pub enum DiskBackendError {
    #[error("Unknown error")]
    Unknown,
}

pub struct DiskFileHandle(u64);

pub enum DiskFileKind {
    File,
    Directory,
}

pub struct DiskFileStat<'a> {
    name: &'a str,
    size: u64,
    kind: DiskFileKind,
}

#[allow(unused_variables)]
pub trait DiskBackend {
    fn open(&mut self, path: &str) -> Result<DiskFileHandle, DiskBackendError>;
    fn close(&mut self, handle: DiskFileHandle) -> Result<(), DiskBackendError>;
    fn read(
        &self,
        handle: DiskFileHandle,
        position: usize,
        buffer: &mut [u8],
    ) -> Result<usize, DiskBackendError>;
    fn write(
        &mut self,
        handle: DiskFileHandle,
        position: usize,
        buffer: &[u8],
    ) -> Result<usize, DiskBackendError>;
    fn parent(&self, handle: DiskFileHandle) -> Option<DiskFileHandle>;
    fn child(&self, handle: DiskFileHandle) -> Option<DiskFileHandle>;
    fn next(&self, handle: DiskFileHandle) -> Option<DiskFileHandle>;
    fn previous(&self, handle: DiskFileHandle) -> Option<DiskFileHandle>;
    fn stat(&self, handle: DiskFileHandle) -> Result<DiskFileStat, DiskBackendError>;
    fn serialize(&mut self, encoder: &mut dyn Encoder) -> Result<(), DiskBackendError>;
}

#[derive(Default)]
pub struct DummyDiskBackend;

impl DiskBackend for DummyDiskBackend {}
