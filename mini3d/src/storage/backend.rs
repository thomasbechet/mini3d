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
    ) -> Result<StorageFileHandle, StorageBackendError> {
        Ok(Default::default())
    }
    fn close(&mut self, handle: StorageFileHandle) -> Result<(), StorageBackendError> {
        Ok(())
    }
    fn encoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Encoder, StorageBackendError> {
        Err(StorageBackendError::Unknown)
    }
    fn decoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Decoder, StorageBackendError> {
        Err(StorageBackendError::Unknown)
    }
    fn info(&self, handle: StorageFileHandle) -> Result<DiskFileInfo, StorageBackendError> {
        Ok(DiskFileInfo {
            name: "",
            kind: DiskFileKind::File,
        })
    }
    fn seek(
        &mut self,
        handle: StorageFileHandle,
        pos: usize,
    ) -> Result<usize, StorageBackendError> {
        Ok(0)
    }
    fn mount(&mut self, path: &str) -> Result<StorageMountHandle, StorageBackendError> {
        Ok(Default::default())
    }
    fn unmount(&mut self, handle: StorageMountHandle) -> Result<(), StorageBackendError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DummyStorageBackend;

impl StorageBackend for DummyStorageBackend {}
