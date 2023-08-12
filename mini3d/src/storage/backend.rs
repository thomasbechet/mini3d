use mini3d_derive::Error;

use crate::{
    define_backend_handle,
    serialize::{Decoder, Encoder},
};

#[derive(Debug, Error)]
pub enum StorageBackendError {
    #[error("Unknown error")]
    Unknown,
}

define_backend_handle!(StorageFileHandle);
define_backend_handle!(StorageMountHandle);

pub enum DiskFileKind {
    File,
    Directory,
}

pub enum DiskFileMode {
    Read,
    Create,
    Append,
}

pub struct DiskFileInfo<'a> {
    name: &'a str,
    kind: DiskFileKind,
}

#[allow(unused_variables)]
pub trait StorageBackend {
    fn open(
        &mut self,
        path: &str,
        mode: DiskFileMode,
    ) -> Result<StorageFileHandle, StorageBackendError>;
    fn close(&mut self, handle: StorageFileHandle) -> Result<(), StorageBackendError>;
    fn encoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Encoder, StorageBackendError>;
    fn decoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Decoder, StorageBackendError>;
    fn info(&self, handle: StorageFileHandle) -> Result<DiskFileInfo, StorageBackendError>;
    fn seek(&mut self, handle: StorageFileHandle, pos: usize)
        -> Result<usize, StorageBackendError>;
    fn mount(&mut self, path: &str) -> Result<StorageMountHandle, StorageBackendError>;
    fn unmount(&mut self, handle: StorageMountHandle) -> Result<(), StorageBackendError>;
}

#[derive(Default)]
pub struct DummyStorageBackend;

impl StorageBackend for DummyStorageBackend {}
